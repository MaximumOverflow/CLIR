mod assembly;
mod metadata;
mod byte_stream;
mod portable_executable;

pub use assembly::*;
pub use metadata::*;
pub use byte_stream::*;
pub use portable_executable::*;

pub use assembly::Assembly;
pub use metadata::tables::Assembly as AssemblyDef;
