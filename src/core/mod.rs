pub mod operation;
pub mod package;
pub mod traits;
pub mod error;
pub mod executor;

pub use operation::Operation;
pub use package::Package;
pub use traits::PackageManager;
pub use error::{PacMagicError, Result};
pub use executor::CommandExecutor;


