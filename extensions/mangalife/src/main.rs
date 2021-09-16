use nepnep::tanoshi_lib::prelude::*;
use std::str::FromStr;

const ID: i64 = 4;
const NAME: &str = "mangalife";
const URL: &str = "https://manga4life.com";
const ICON_URL: &str = "https://manga4life.com/media/favicon.png";

#[derive(Default)]
pub struct Mangalife;

nepnep::generate!(Mangalife);
