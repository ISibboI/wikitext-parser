pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    /// Found a second root section, but only one is allowed.
    SecondRootSection {
        /// The label of the second root section.
        label: String,
    },

    /// Found a section at a level that is deeper than supported.
    SectionLevelTooDeep {
        /// The too deep level.
        level: usize,
    },
}
