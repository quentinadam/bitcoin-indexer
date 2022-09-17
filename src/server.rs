use hyper::{
    body::HttpBody,
    service::{make_service_fn, service_fn},
    Body, Method, Request, Response, StatusCode,
};
use regex::Regex;
use std::{borrow::Cow, collections::HashMap, convert::Infallible, future::Future, net::SocketAddr, pin::Pin, sync::Arc};

pub struct DynamicUri {
    regex: Regex,
    parameters: HashMap<String, usize>,
}

fn escape_regex<'t>(text: &'t str) -> Cow<'t, str> {
    let re = Regex::new(r"([\.\+\*\?\^\$\(\)\[\]\{\}\|\\])").unwrap();
    re.replace_all(text, |caps: &regex::Captures| format!(r"\{}", &caps[1]))
}

impl DynamicUri {
    pub fn new(uri: &str) -> Self {
        let regex = Regex::new(r"\{([^\{]*)\}").unwrap();
        let mut offset = 0;
        let mut index = 1;
        let mut chunks = Vec::new();
        let mut parameters = HashMap::new();
        for occurence in regex.find_iter(uri) {
            let range = occurence.range();
            let name = &uri[range.start + 1..range.end - 1];
            assert!(parameters.get(name).is_none());
            parameters.insert(name.to_string(), index);
            chunks.push(escape_regex(&uri[offset..range.start]));
            chunks.push(Cow::from("([^/]*)"));
            offset = range.end;
            index += 1;
        }
        chunks.push(escape_regex(&uri[offset..]));
        let regex = Regex::new(&format!("^{}$", chunks.join(""))).unwrap();
        Self { regex, parameters }
    }

    pub fn check(&self, uri: &str) -> Option<HashMap<String, String>> {
        if let Some(captures) = self.regex.captures(uri) {
            let mut parameters = HashMap::new();
            for (name, &index) in &self.parameters {
                parameters.insert(name.clone(), captures[index].to_string());
            }
            Some(parameters)
        } else {
            None
        }
    }
}

type BoxedResponseFuture = Pin<Box<dyn Send + Future<Output = Response<Body>>>>;
type BoxedHandler<S> = Box<dyn Fn(Request<Body>, HashMap<String, String>, Vec<u8>, Arc<S>) -> BoxedResponseFuture + Send + Sync>;
type BoxedDefaultHandler<S> = Box<dyn Fn(Request<Body>, Vec<u8>, Arc<S>) -> BoxedResponseFuture + Send + Sync>;

struct Route<S> {
    method: Method,
    dynamic_uri: DynamicUri,
    handler: BoxedHandler<S>,
}

impl<S> Route<S> {
    fn new(method: Method, uri: &str, handler: BoxedHandler<S>) -> Self {
        Self {
            method,
            dynamic_uri: DynamicUri::new(uri),
            handler,
        }
    }

    pub fn check(&self, request: &Request<Body>) -> Option<HashMap<String, String>> {
        if request.method() == self.method {
            self.dynamic_uri.check(request.uri().path())
        } else {
            None
        }
    }
}

pub struct Router<S> {
    default: Option<BoxedDefaultHandler<S>>,
    routes: Vec<Route<S>>,
    state: Arc<S>,
}

impl<S> Router<S> {
    pub fn new(state: Arc<S>) -> Self {
        Self {
            default: None,
            routes: Vec::new(),
            state,
        }
    }

    pub fn get<H, F>(&mut self, uri: &str, handler: H)
    where
        H: 'static + Fn(Request<Body>, HashMap<String, String>, Vec<u8>, Arc<S>) -> F + Send + Sync,
        F: 'static + Future<Output = Response<Body>> + Send,
    {
        self.routes.push(Route::new(
            Method::GET,
            uri,
            Box::new(move |request, parameters, body, state| Box::pin(handler(request, parameters, body, state))),
        ));
    }

    pub fn post<H, F>(&mut self, uri: &str, handler: H)
    where
        H: 'static + Fn(Request<Body>, HashMap<String, String>, Vec<u8>, Arc<S>) -> F + Send + Sync,
        F: 'static + Future<Output = Response<Body>> + Send,
    {
        self.routes.push(Route::new(
            Method::POST,
            uri,
            Box::new(move |request, parameters, body, state| Box::pin(handler(request, parameters, body, state))),
        ));
    }

    pub fn default<H, F>(&mut self, handler: H)
    where
        H: 'static + Fn(Request<Body>, Vec<u8>, Arc<S>) -> F + Send + Sync,
        F: 'static + Future<Output = Response<Body>> + Send,
    {
        self.default = Some(Box::new(move |request, body, state| Box::pin(handler(request, body, state))));
    }

    pub async fn route(&self, request: Request<Body>, body: Vec<u8>) -> Response<Body> {
        println!("{} {}", request.method(), request.uri());
        for route in &self.routes {
            if let Some(mut parameters) = route.check(&request) {
                if let Some(query) = request.uri().query() {
                    for part in query.split('&') {
                        let parts: Vec<&str> = part.split('=').collect();
                        if parts.len() == 2 {
                            parameters.insert(parts[0].to_string(), parts[1].to_string());
                        }
                    }
                }
                return (route.handler)(request, parameters, body, self.state.clone()).await;
            }
        }
        match &self.default {
            Some(default) => default(request, body, self.state.clone()).await,
            None => Response::builder().status(StatusCode::NOT_FOUND).body("Not Found".into()).unwrap(),
        }
    }
}

pub struct Server<S> {
    router: Router<S>,
}

impl<S: 'static + Send + Sync> Server<S> {
    pub fn new(state: Arc<S>) -> Self {
        Self {
            router: Router::new(state),
        }
    }

    pub fn get<H, F>(&mut self, uri: &str, handler: H)
    where
        H: 'static + Fn(Request<Body>, HashMap<String, String>, Vec<u8>, Arc<S>) -> F + Send + Sync,
        F: 'static + Future<Output = Response<Body>> + Send,
    {
        self.router.get(uri, handler);
    }

    pub fn post<H, F>(&mut self, uri: &str, handler: H)
    where
        H: 'static + Fn(Request<Body>, HashMap<String, String>, Vec<u8>, Arc<S>) -> F + Send + Sync,
        F: 'static + Future<Output = Response<Body>> + Send,
    {
        self.router.post(uri, handler);
    }

    pub fn default<H, F>(&mut self, handler: H)
    where
        H: 'static + Fn(Request<Body>, Vec<u8>, Arc<S>) -> F + Send + Sync,
        F: 'static + Future<Output = Response<Body>> + Send,
    {
        self.router.default(handler);
    }

    pub async fn run(self, address: SocketAddr) {
        let router = Arc::new(self.router);
        let make_service = make_service_fn(|_| {
            let router = router.clone();
            async {
                Ok::<_, Infallible>(service_fn(move |mut request: Request<Body>| {
                    let router = router.clone();
                    async move {
                        let mut body: Vec<u8> = Vec::new();
                        while let Some(chunk) = request.body_mut().data().await {
                            body.extend_from_slice(&chunk.unwrap());
                        }
                        let result: Result<Response<Body>, Infallible> = Ok(router.route(request, body).await);
                        result
                    }
                }))
            }
        });
        let server = hyper::Server::bind(&address);
        server.serve(make_service).await.unwrap();
    }
}
