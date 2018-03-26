macro_rules! try_multiple_time {
    ($e:expr) => (
        {
            let mut error_counter = 0;
            let mut res = $e;
            while res.is_err() {
                ::std::thread::sleep(::std::time::Duration::from_millis(10));
                error_counter += 1;
                if error_counter > 10 {
                    break;
                }
                res = $e;
            }
            res
        }
    )
}
