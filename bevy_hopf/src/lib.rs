//! A Bevy plugin for generating Hopf fibration meshes.
#![deny(clippy::all)]
#![warn(clippy::cargo)]
#![warn(clippy::complexity)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![warn(clippy::perf)]
#![warn(missing_debug_implementations)]
#![warn(missing_docs)]
#![allow(clippy::many_single_char_names)]

/// A struct and methods for generating a Hopf mesh.
pub mod hopf;

/// A struct and methods for generating a Hopf fibration.
#[derive(Debug)]
pub struct HopfPlugin;

use bevy::app::App;
use bevy::app::Plugin;

impl Plugin for HopfPlugin {
    fn build(&self, _app: &mut App) {
        // add things to your app here
    }
}
