// BSL 1.0 License

/// Operations that can be used to composite two surfaces together.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CompositeOperation {
    Src,
    Over,
}
