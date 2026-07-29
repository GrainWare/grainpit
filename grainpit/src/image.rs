use rand::RngExt;
use zune_core::options::EncoderOptions;
use zune_image::{codecs::jpeg::JpegEncoder, traits::OperationsTrait};
use zune_imageprocs::{invert::Invert, sobel::Sobel};

pub fn gen_image() -> Vec<u8> {
    let img_x = 88;
    let img_y = 31;
    let encoder = JpegEncoder::new_with_options(EncoderOptions::default().set_quality(5));

    let mut pixels = [0; (88 * 31) * 3];
    rand::rng().fill(&mut pixels);

    let mut image = zune_image::image::Image::from_u8(
        &pixels,
        img_x,
        img_y,
        zune_core::colorspace::ColorSpace::RGB,
    );

    let sobel = Sobel::new();
    sobel.execute(&mut image).unwrap();
    if rand::random_bool(0.5) {
        let invert = Invert::new();
        invert.execute(&mut image).unwrap();
    }

    let mut output = vec![];
    image.write_with_encoder(encoder, &mut output).unwrap();
    output
}
