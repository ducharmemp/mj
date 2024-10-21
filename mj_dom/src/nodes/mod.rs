use std::collections::{HashMap, VecDeque};

use ecow::EcoString;
use html5ever::QualName;
use ordermap::OrderMap;
use stakker::{call, lazy, ret, ret_do, ret_some_do, ret_some_to, ret_to, stop, Actor, Ret, CX};

use crate::parser::NodeId;

pub mod document;
pub mod dom_entry;
