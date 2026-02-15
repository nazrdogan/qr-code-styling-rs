//! Plugins for extending QR code functionality.

pub mod border;

pub use border::{
    BorderDecoration, BorderOptions, BorderPlugin, DecorationType, Position, QRBorderOptions,
};
