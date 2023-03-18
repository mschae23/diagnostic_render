#![allow(unused_imports)] // for pretty:assertions::{assert_eq, assert_ne}

use crate::diagnostic::{AnnotationStyle, Severity};
use crate::file::SimpleFile;
use pretty_assertions::{assert_eq, assert_ne};
use super::*;

mod singleline;
mod ending;
mod starting;

mod vertical_offset;
