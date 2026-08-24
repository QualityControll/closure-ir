use closure_ir::{call, compile_closure};

#[test]
fn test_array_indexing_i32() { let compiled=compile_closure!(|values:[i32;4]|->i32{values[0]+values[1]+values[2]+values[3]}); assert_eq!(call!(compiled,[1,2,3,4]),10); }
#[test]
fn test_array_indexing_f64() { let compiled=compile_closure!(|values:[f64;3]|->f64{values[0]*values[2]}); assert_eq!(call!(compiled,[2.0,4.0,5.0]),10.0); }
#[test]
fn test_nested_array_indexing() { let compiled=compile_closure!(|values:[[i32;2];2]|->i32{values[0][1]+values[1][0]}); assert_eq!(call!(compiled,[[1,2],[3,4]]),5); }
#[test]
fn test_usize_literal() { let compiled=compile_closure!(||->usize{4usize}); assert_eq!(call!(compiled),4usize); }
