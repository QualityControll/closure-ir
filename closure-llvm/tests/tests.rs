
use closure_llvm::{
    call,
    compile_closure,
    CompileType,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_boolean() {
        let compiled = compile_closure!(|b: bool| -> bool { b });
        let val = false;
        let result = call!(compiled, val);
        assert_eq!(result, false);
    }

    #[test]
    fn test_rectangle_width() {
        #[repr(C)]
        #[derive(Debug, Clone, Copy, CompileType)]
        struct Point {
            x: f64,
            y: f64,
        }

        #[repr(C)]
        #[derive(Debug, Clone, Copy, CompileType)]
        struct Rectangle {
            top_left: Point,
            bottom_right: Point,
        }

        let compiled = compile_closure!(|r: Rectangle| -> f64 { r.bottom_right.x - r.top_left.x });

        let rectangle = Rectangle {
            top_left: Point { x: 10.0, y: 20.0 },
            bottom_right: Point { x: 30.0, y: 50.0 },
        };

        let result = call!(compiled, rectangle);

        assert_eq!(result, 20.0);
    }
}