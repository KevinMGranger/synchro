mod cloud_filter;
mod real_main;
mod util;

use anyhow::Result;

trait MainTrait {
    fn foo(&self) -> &str;
}

trait DispatcherTrait {
    fn main(&self) -> Option<&dyn MainTrait>;
}

// or is just returning a pointer to the method better?

fn main() -> Result<()> {
    real_main::main()

    // let binkus = "asdf";

    // dbg!(&binkus[3..5]);

    // Ok(())
}
