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

#[inline]
pub fn to_grid(coords: &::na::Vector3<f32>, scale: f32) -> ::na::Vector3<isize> {
    ::na::Vector3::<isize>::from_iterator(coords.iter().map(|&c| (c / scale) as isize))
}

#[inline]
pub fn to_world(coords: &::na::Vector3<isize>, scale: f32) -> ::na::Vector3<f32> {
    let mut res = ::na::Vector3::new(scale, scale, scale) * 0.5;
    for i in 0..3 {
        res[i] += (coords[i] as f32) * scale;
    }
    res
}
