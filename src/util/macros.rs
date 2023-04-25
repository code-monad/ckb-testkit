#[macro_export]
macro_rules! assert_result_eq {
    ($left:expr, $right:expr) => {
        if $left.is_err() && $right.is_err() {
            let left_ = $left.as_ref().map_err(|err| err.to_string());
            let right_ = $right.as_ref().map_err(|err| err.to_string());
            let left_raw_err = left_.as_ref().unwrap_err();
            let right_raw_err = right_.as_ref().unwrap_err();
            if !(left_raw_err.contains(right_raw_err) || right_raw_err.contains(left_raw_err)) {
                assert_eq!(Result::<(), &str>::Err(left_raw_err), Result::<(), &str>::Err(right_raw_err));
            }
        } else {
            let left_ = $left.as_ref().map_err(|err| err.to_string());
            let right_ = $right.as_ref().map_err(|err| err.to_string());
            assert_eq!(left_, right_);
        }
    };
    ($left:expr, $right:expr,) => {
        $crate::assert_result_eq!($left, $right);
    };
    ($left:expr, $right:expr, $($arg:tt)+) => {
        if $left.is_err() && $right.is_err() {
            let left_ = $left.as_ref().map_err(|err| err.to_string());
            let right_ = $right.as_ref().map_err(|err| err.to_string());
            let left_raw_err = left_.as_ref().unwrap_err();
            let right_raw_err = right_.as_ref().unwrap_err();
            if !(left_raw_err.contains(right_raw_err) || right_raw_err.contains(left_raw_err)) {
                assert_eq!(Result::<(), &str>::Err(left_raw_err), Result::<(), &str>::Err(right_raw_err), $($arg)+);
            }
        } else {
            let left_ = $left.as_ref().map_err(|err| err.to_string());
            let right_ = $right.as_ref().map_err(|err| err.to_string());
            assert_eq!(left_, right_, $($arg)+);
        }
    }
}
