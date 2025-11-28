pub mod error;
pub mod executor;
pub mod operation;
pub mod package;
pub mod traits;

pub use error::{PacMagicError, Result};
pub use executor::CommandExecutor;
pub use operation::{Operation, OperationOptions};
pub use package::Package;
pub use traits::PackageManager;
