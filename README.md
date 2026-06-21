# `bevy_softbody2d`

[![License](https://img.shields.io/badge/license-MIT%2FApache-blue.svg)](https://github.com/Katsutoshii/bevy_softbody2d#license)
[![Crates.io](https://img.shields.io/crates/v/bevy_softbody2d.svg)](https://crates.io/crates/bevy_softbody2d)
[![Docs](https://docs.rs/bevy_softbody2d/badge.svg)](https://docs.rs/bevy_softbody2d/latest/bevy_softbody2d/)

## Usage

`bevy_softbody2d` provides a component `SoftBody2d` that will construct a SoftBody mesh around the given nodes.
The mesh will act as a shrink-wrapped bag around the nodes and automatically updates to the nodes position.
This means that users can specify any physics they like for the nodes.
