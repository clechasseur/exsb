use wiremock::{Match, Request};

pub struct NotMatcher(Box<dyn Match>);

impl NotMatcher {
    pub fn from_matcher<M: Match + 'static>(matcher: M) -> Self {
        Self(Box::new(matcher))
    }
}

impl Match for NotMatcher {
    fn matches(&self, request: &Request) -> bool {
        !self.0.matches(request)
    }
}

pub fn not<M: Match + 'static>(matcher: M) -> NotMatcher {
    NotMatcher::from_matcher(matcher)
}
