use crate::{layouts::*, views::*};
use dioxus::prelude::*;

#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
pub enum Route {
    #[layout(BaseLayout)]
    #[route("/")]
    MainPage {},
}
