mod cloud_filter;
mod real_main;
mod util;

use anyhow::Result;

// Since various callbacks are optional, 
// I'm experimenting with how one might choose which ones to implement.
trait MainTrait {
    fn foo(&self) -> &str;
}

trait DispatcherTrait {
// or is just returning a pointer to the method better?
    fn main(&self) -> Option<&dyn MainTrait>;
}


fn main() -> Result<()> {
    real_main::main()

    // let binkus = "asdf";

    // dbg!(&binkus[3..5]);

    // Ok(())
}
