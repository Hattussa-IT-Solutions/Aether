pub mod string;
pub mod list;
pub mod map;
pub mod set;
pub mod math;
pub mod io;
pub mod net;
pub mod json;
pub mod time;
pub mod tensor;

use crate::interpreter::environment::Environment;

/// Register all standard library functions.
pub fn register_all(env: &mut Environment) {
    string::register_string_methods(env);
    math::register_math(env);
    io::register_io(env);
    time::register_time(env);
    set::register_set_methods(env);
    list::register_list_methods(env);
    json::register_json(env);
    tensor::register_tensor(env);
}
