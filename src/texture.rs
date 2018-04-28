use rand::Rand;
use rand::distributions::{IndependentSample, Range};

pub fn generate_texture(
    width: u32,
    height: u32,
    layers: u32,
    filter: ::image::FilterType,
    absissa_continuous: bool,
) -> ::image::ImageBuffer<::image::Luma<u8>, Vec<u8>> {
    let mut rng = ::rand::thread_rng();
    let mut deepness = layers - 1;

    let tmp_width = if absissa_continuous {
        width * 2
    } else {
        width
    };

    let mut images: Vec<::image::ImageBuffer<::image::Luma<u8>, _>>= vec![];
    while deepness != 0 {
        deepness -= 1;
        let factor = 2_u32.pow(deepness);

        let sub_image_width = tmp_width / factor;
        let sub_image_height = height / factor;

        let data = (0..sub_image_width*sub_image_height)
            // IDEA: other distributions for example: [0..1]^2 * 255
            .map(|_| u8::rand(&mut rng))
            .collect::<Vec<_>>();

        let image = ::image::ImageBuffer::from_vec(sub_image_width, sub_image_height, data).unwrap();
        let image = ::image::imageops::resize(&image, tmp_width, height, filter);

        images.push(image);
    }

    let x_delta = if absissa_continuous {
        width/2
    } else {
        0
    };

    ::image::ImageBuffer::from_fn(width, height, |x, y| {
        let data = images
            .iter()
            .map(|image| image[(x + x_delta, y)].data[0] as u32)
            .sum::<u32>() / images.len() as u32;
        ::image::Luma { data: [data as u8] }
    })
}

/// Add itself to flip horizontal and flip vertical and rotate 90 and rotate 270 to make it as unlocal as possible
///
/// image must be a square
pub fn unlocal(
    mut image: ::image::ImageBuffer<::image::Luma<u8>, Vec<u8>>,
    mut iteration: usize,
) -> ::image::ImageBuffer<::image::Luma<u8>, Vec<u8>> {
    assert_eq!(image.width(), image.height());

    let mut rng = ::rand::thread_rng();
    while iteration != 0 {
        iteration -= 1;

        let transform = match Range::new(0, 4).ind_sample(&mut rng) {
            0 => ::image::imageops::flip_horizontal(&image),
            1 => ::image::imageops::flip_vertical(&image),
            2 => ::image::imageops::rotate90(&image),
            3 => ::image::imageops::rotate270(&image),
            _ => unreachable!(),
        };

        image = ::image::ImageBuffer::from_fn(image.width(), image.height(), |x, y| {
            let data = (image[(x, y)].data[0] as u32 + transform[(x, y)].data[0] as u32) / 2;
            ::image::Luma { data: [data as u8] }
        });
    }

    image
}
