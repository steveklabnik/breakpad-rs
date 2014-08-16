extern crate breakpad_client;

use breakpad_client::ExceptionHandler;

fn main() {
    breakpad_client::catch_task_failure();
    let eh = ExceptionHandler::new(&Path::new("/tmp"));
    fail!();
}
