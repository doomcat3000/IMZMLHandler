extern crate glutin_window;
extern crate graphics;
extern crate nfd;
extern crate opengl_graphics;
extern crate piston;

use crate::enzymepresence::*;
use crate::imzml_types::*;
use crate::parsers::*;
use glutin_window::GlutinWindow as Window;
use graphics::rectangle::square;
use graphics::Image;
use image::DynamicImage;
use image::{GenericImage, GenericImageView, ImageBuffer, Pixel, Pixels, RgbImage};
use nfd::Response;
use opengl_graphics::*;
use opengl_graphics::{GlGraphics, OpenGL};
use piston::event_loop::{EventSettings, Events};
use piston::input::{ButtonEvent, Event, RenderEvent};
use piston::window::WindowSettings;
use piston::Button::Keyboard;
use piston::ButtonState::Release;
use piston::Key;
use std::path::Path;
pub fn translatetoscreencords(
    (mz, intensity): (f32, f32),
    spec: &OSpectrum,
    screenmin: f32,
    screenmax: f32,
    midx: f32,
    midy: f32,
    intensityscale: f32,
) -> [f64; 2] {
    [
        ((((((spec.max - spec.min) * mz) + spec.min) - screenmin) / (screenmax - screenmin))
            * (midx * 2.)) as f64,
        ((midy * 2.)
            - (((intensity * intensityscale) * (((midy * 6.) / 7.) - (midy / 7.))) + ((midy) / 7.)))
            as f64,
    ]
}
pub fn start() {
    use graphics::*;
    let opengl = OpenGL::V3_2;
    let mut window: Window = WindowSettings::new("GN's OMS Visualizor", (1024, 768))
        .graphics_api(opengl)
        .exit_on_esc(true)
        .build()
        .unwrap();
    let mut graphics: GlGraphics = GlGraphics::new(opengl);
    let mut events = Events::new(EventSettings::new());
    let mut event_count = 0;
    let mut mousex = 0.;
    let mut mousey = 0.;
    let mut mid_x = 0.;
    let mut mid_y = 0.;
    let start_time = std::time::SystemTime::now();
    let mut glyph_cache = GlyphCache::new(
        "assets/GoldleafBoldPersonalUseBold-eZ4dO.ttf",
        (),
        TextureSettings::new(),
    )
    .unwrap();
    let mut glyph_cache2 = GlyphCache::new(
        "assets/PlayfairDisplayRegular-ywLOY.ttf",
        (),
        TextureSettings::new(),
    )
    .unwrap();
    let mut firstsel = true;
    let mut stagenum = 0;
    let mut selectedspec = OSpectrum::new(0., Vec::new(), 0., 0.);
    let mut mapinmemory: OptimizedMSMap = OptimizedMSMap::new(0, 0);
    let mut histaminrange = 0.;
    let mut histamaxrange = 3000.;
    let mut currentminrange = 0.;
    let mut currentmaxrange = 3000.;
    let mut intensityscale = 1.;
    let mut histaspecselection = (1, 1);
    let mut imagemem = Image::new().rect(square(0.0, 0.0, 0.0));
    let mut texturemem = Texture::new(0, 0, 0);
    let (mut histatempmin, mut histatempmax): (f32, f32) = (0., 0.);
    let (mut currenttempmin, mut currenttempmax): (f32, f32) = (0., 0.);
    let mut intensitytempscale = 0.;
    let mut pressinginsidebox = false;
    let mut pressinginbottombox = false;
    let mut pressinginbottommiddlebox = false;
    while let Some(evt) = events.next(&mut window) {
        let evt: Event = evt;
        event_count += 1;
        if (false && firstsel) {
            let result = nfd::open_file_dialog(Some("oms"), None).unwrap_or_else(|e| {
                panic!(e);
            });
            match result {
                Response::Okay(file_path) => println!("File path = {:?}", file_path),
                Response::OkayMultiple(files) => println!("Files {:?}", files),
                Response::Cancel => println!("Canceled"),
            }
            firstsel = false;
        }
        use piston::input::*;
        match evt {
            // Mouse *position* events. (piston_window's names are badly misleading here. These are coords within the window, not movements.)
            Event::Input(Input::Move(Motion::MouseCursor(mousepos_args)), _timestamp_not_used) => {
                mousex = mousepos_args[0];
                mousey = mousepos_args[1];
            }
            Event::Input(Input::Button(button_args), _timestamp_not_used) => {
                if (button_args.button == Button::Mouse(MouseButton::Left)) {
                    if let Some(args) = evt.button_args() {
                        if !(args.state == Release) {
                            if (stagenum == 1) {
                                if (mousey > (mid_y * 2.) - (mid_y / 7.)) {
                                    if (mousey > (mid_y * 2.) - (mid_y / 14.)) {
                                    } else {
                                        pressinginbottommiddlebox = true;
                                        histatempmin = mousex as f32;
                                    }
                                } else if (mousex > ((mid_y * 2.) - (mid_y / 7.))) {
                                }
                            }
                        } else {

                            //IBD and IMZML
                            if (stagenum == 0) {
                                if ((mousex > mid_x - (mid_x / 4. - (mid_x / 3.)))
                                    && (mousex < mid_x + ((mid_x / 3.) + mid_x / 4.)))
                                    && ((mousey > (mid_y - (mid_x / 4.)))
                                        && (mousey < (mid_y + (mid_x / 4.))))
                                {
                                    let result = nfd::open_file_dialog(Some("oms"), None)
                                        .unwrap_or_else(|e| {
                                            panic!(e);
                                        });
                                    let mut omspath: String = "".to_string();
                                    match result {
                                        Response::Okay(file_path) => omspath = file_path,
                                        Response::OkayMultiple(files) => omspath = files[0].clone(),
                                        Response::Cancel => return,
                                    }
                                    mapinmemory =
                                        (read_oms_file(Path::new(&omspath).to_path_buf()));
                                    let img = bluegreenimagefromOMS(
                                        &mapinmemory,
                                        currentminrange,
                                        currentmaxrange,
                                    );
                                    imagemem = Image::new().rect([
                                        0.,
                                        0.,
                                        img.dimensions().0 as f64,
                                        img.dimensions().1 as f64,
                                    ]);
                                    img.save(Path::new("imgstorage/memimage.png"));
                                    texturemem = Texture::from_path(
                                        Path::new("imgstorage/memimage.png"),
                                        &TextureSettings::new(),
                                    )
                                    .unwrap();
                                    // Signifies that the map was loaded to memory
                                    stagenum = 1;
                                }
                                //OMS
                                if ((mousex > mid_x - (mid_x / 4. + (mid_x / 3.)))
                                    && (mousex < mid_x - ((mid_x / 3.) - mid_x / 4.)))
                                    && ((mousey > (mid_y - (mid_x / 4.)))
                                        && (mousey < (mid_y + (mid_x / 4.))))
                                {
                                    let result = nfd::open_file_dialog(Some("imzml"), None)
                                        .unwrap_or_else(|e| {
                                            panic!(e);
                                        });
                                    let mut imzmlpath: String = "".to_string();
                                    match result {
                                        Response::Okay(file_path) => imzmlpath = file_path,
                                        Response::OkayMultiple(files) => {
                                            imzmlpath = files[0].clone()
                                        }
                                        Response::Cancel => return,
                                    }
                                    let result = nfd::open_file_dialog(Some("ibd"), None)
                                        .unwrap_or_else(|e| {
                                            panic!(e);
                                        });
                                    let mut ibdpath: String = "".to_string();
                                    match result {
                                        Response::Okay(file_path) => ibdpath = file_path,
                                        Response::OkayMultiple(files) => ibdpath = files[0].clone(),
                                        Response::Cancel => return,
                                    }
                                    mapinmemory = (readoptimizedMSmap(
                                        Path::new(&ibdpath).to_path_buf(),
                                        Path::new(&imzmlpath).to_path_buf(),
                                        &mut graphics,
                                        &mut window,
                                        &mut events,
                                    ));
                                    let img = bluegreenimagefromOMS(
                                        &mapinmemory,
                                        currentminrange,
                                        currentmaxrange,
                                    );
                                    imagemem = Image::new().rect([
                                        0.,
                                        0.,
                                        img.dimensions().0 as f64,
                                        img.dimensions().1 as f64,
                                    ]);
                                    img.save(Path::new("imgstorage/memimage.png"));
                                    texturemem = Texture::from_path(
                                        Path::new("imgstorage/memimage.png"),
                                        &TextureSettings::new(),
                                    )
                                    .unwrap();
                                    // Signifies that the map was loaded to memory
                                    stagenum = 1;
                                }
                            } else {
                                if(pressinginbottommiddlebox){
                                    let histaminrange2 = histaminrange + (((histatempmin) / ((mid_x * 2.) as f32)) * (histamaxrange - histaminrange));
                                    let histamaxrange2 = histaminrange + (((mousex as f32) / ((mid_x * 2.) as f32)) * (histamaxrange - histaminrange));
                                    histaminrange = histaminrange2.clone();
                                    histamaxrange = histamaxrange2.clone();
                                    pressinginbottommiddlebox = false;
                                }
                                let adjustedmx = (mousex
                                    - (((mid_x * 2.)
                                        - (imagemem.rectangle.unwrap()[2]
                                            * (mid_y / imagemem.rectangle.unwrap()[3])))
                                        / 2.));
                                if (mousey < mid_y) {
                                    if ((((adjustedmx / mid_y) * (imagemem.rectangle.unwrap()[3]))
                                        >= 0.)
                                        && (((adjustedmx / mid_y)
                                            * (imagemem.rectangle.unwrap()[3]))
                                            < (imagemem.rectangle.unwrap()[2])))
                                        && ((((mousey / mid_y) * (imagemem.rectangle.unwrap()[3]))
                                            >= 0.)
                                            && (((mousey / mid_y)
                                                * (imagemem.rectangle.unwrap()[3]))
                                                < (imagemem.rectangle.unwrap()[3])))
                                    {
                                        histaspecselection = (
                                            ((adjustedmx / mid_y)
                                                * (imagemem.rectangle.unwrap()[3]))
                                                .floor()
                                                as i64,
                                            ((mousey / mid_y) * (imagemem.rectangle.unwrap()[3]))
                                                .floor()
                                                as i64,
                                        );
                                    }
                                }
                                selectedspec = (&mapinmemory)
                                    .arrayspec
                                    .get(
                                        histaspecselection.0 as usize,
                                        histaspecselection.1 as usize,
                                    )
                                    .unwrap()
                                    .to_owned();
                            }
                        }
                    }
                }
            }
            _ => {
                if let Some(args) = evt.render_args() {
                    const WHITE: [f32; 4] = [1.0, 1.0, 1.0, 1.0];
                    const BLACK: [f32; 4] = [0.0, 0.0, 0.0, 1.0];
                    const GRAY: [f32; 4] = [0.7, 0.7, 0.7, 1.0];
                    const LGRAY: [f32; 4] = [0.85, 0.85, 0.85, 1.0];
                    const GREEN: [f32; 4] = [0.0, 1.0, 0.0, 1.0];
                    (mid_x, mid_y) = (args.window_size[0] / 2.0, args.window_size[1] / 2.0);
                    graphics.draw(args.viewport(), |c, gl| {
                        clear(WHITE, gl); // Clear the screen.
                        if (stagenum == 1) {
                            for peak in &selectedspec.peeks {
                                line_from_to(
                                    BLACK,
                                    2.0,
                                    translatetoscreencords(
                                        (
                                            (peak.relativemztominandmax as f32 / 4294967295.)
                                                + ((((peak.intensityrelativetomax as f32 / 65535.)
                                                    * &selectedspec.highestintensity)
                                                    / (peak.slopepermzover10 as f32 * 10.))
                                                    / (&selectedspec.max - selectedspec.min)),
                                            0.,
                                        ),
                                        &selectedspec,
                                        histaminrange,
                                        histamaxrange,
                                        mid_x as f32,
                                        mid_y as f32,
                                        intensityscale,
                                    ),
                                    translatetoscreencords(
                                        (
                                            peak.relativemztominandmax as f32 / 4294967295.,
                                            peak.intensityrelativetomax as f32 / 65535.,
                                        ),
                                        &selectedspec,
                                        histaminrange,
                                        histamaxrange,
                                        mid_x as f32,
                                        mid_y as f32,
                                        intensityscale,
                                    ),
                                    c.transform,
                                    gl,
                                );
                                line_from_to(
                                    BLACK,
                                    2.0,
                                    translatetoscreencords(
                                        (
                                            (peak.relativemztominandmax as f32 / 4294967295.)
                                                - ((((peak.intensityrelativetomax as f32 / 65535.)
                                                    * &selectedspec.highestintensity)
                                                    / (peak.slopepermzover10 as f32 * 10.))
                                                    / (&selectedspec.max - selectedspec.min)),
                                            0.,
                                        ),
                                        &selectedspec,
                                        histaminrange,
                                        histamaxrange,
                                        mid_x as f32,
                                        mid_y as f32,
                                        intensityscale,
                                    ),
                                    translatetoscreencords(
                                        (
                                            peak.relativemztominandmax as f32 / 4294967295.,
                                            peak.intensityrelativetomax as f32 / 65535.,
                                        ),
                                        &selectedspec,
                                        histaminrange,
                                        histamaxrange,
                                        mid_x as f32,
                                        mid_y as f32,
                                        intensityscale,
                                    ),
                                    c.transform,
                                    gl,
                                );
                            }
                            let rectangle4 = Rectangle::new([1.0, 0.66, 0.66, 1.0]);
                            let rectangle3 = Rectangle::new(WHITE);
                            let rectangle = Rectangle::new(BLACK);
                            rectangle3.draw(
                                [0., 0., mid_x * 2., mid_y * (8. / 7.)],
                                &Default::default(),
                                c.transform.trans(0., 0.),
                                gl,
                            );
                            rectangle.draw(
                                [0., 0., mid_x * 2., mid_y],
                                &Default::default(),
                                c.transform.trans(0., 0.),
                                gl,
                            );
                            let rectangle2 = Rectangle::new([1.0, 0., 0., 1.0]);
                            line_from_to(
                                BLACK,
                                2.0,
                                [0., (mid_y) + (mid_y / 15.)],
                                [mid_x * 2., (mid_y) + (mid_y / 15.)],
                                c.transform,
                                gl,
                            );
                            line_from_to(
                                BLACK,
                                2.0,
                                [0., (mid_y * 2.) - (mid_y / 7.)],
                                [mid_x * 2., (mid_y * 2.) - (mid_y / 7.)],
                                c.transform,
                                gl,
                            );
                            line_from_to(
                                GRAY,
                                1.0,
                                [0., (mid_y * 2.) - (mid_y / 14.)],
                                [mid_x * 2., (mid_y * 2.) - (mid_y / 14.)],
                                c.transform,
                                gl,
                            );
                            line_from_to(
                                BLACK,
                                2.0,
                                [(mid_x * 2.) - (mid_y / 7.), mid_y + (mid_y / 15.)],
                                [(mid_x * 2.) - (mid_y / 7.), (mid_y * 2.) - (mid_y / 7.)],
                                c.transform,
                                gl,
                            );
                            imagemem.draw(
                                &texturemem,
                                &DrawState::default(),
                                c.transform
                                    .scale(
                                        mid_y / imagemem.rectangle.unwrap()[3],
                                        mid_y / imagemem.rectangle.unwrap()[3],
                                    )
                                    .trans(
                                        (((mid_x * 2.)
                                            - (imagemem.rectangle.unwrap()[2]
                                                * (mid_y / imagemem.rectangle.unwrap()[3])))
                                            / 2.)
                                            / (mid_y / imagemem.rectangle.unwrap()[3]),
                                        0.,
                                    ),
                                gl,
                            );
                            rectangle2.draw(
                                [
                                    0.,
                                    0.,
                                    mid_y / imagemem.rectangle.unwrap()[3],
                                    mid_y / imagemem.rectangle.unwrap()[3],
                                ],
                                &Default::default(),
                                c.transform.trans(
                                    (histaspecselection.0 as f64
                                        * (mid_y / imagemem.rectangle.unwrap()[3]))
                                        + (((mid_x * 2.)
                                            - (imagemem.rectangle.unwrap()[2]
                                                * (mid_y / imagemem.rectangle.unwrap()[3])))
                                            / 2.),
                                    histaspecselection.1 as f64
                                        * (mid_y / imagemem.rectangle.unwrap()[3]),
                                ),
                                gl,
                            );
                            if (pressinginbottommiddlebox) {
                                let mut offs = 0.;
                                if (histatempmin > mousex as f32){
                                    offs = mousex;
                                }else{
                                    offs = histatempmin as f64;
                                }
                                rectangle4.draw(
                                    [
                                        0.,
                                        0.,
                                        (mousex - histatempmin as f64).abs(),
                                        mid_y / 14.,
                                    ],
                                    &Default::default(),
                                    c.transform.trans(
                                        offs,
                                        (mid_y * 2.) - mid_y / 7.,
                                    ),
                                    gl,
                                );
                            }
                            let mut itar: f64 = 0.5;
                            while itar < 10. {
                            let inputnum = histaminrange + ((histamaxrange - histaminrange) * ((itar / 10.) as f32));
                            let mut inputnumrep = inputnum.clone();
                            let mut itar2 = 0;
                            while (inputnumrep >= 0.000001){
                                inputnumrep *= 0.1;
                                itar2 += 1;
                            }
                            itar2 -= 6;
                            text::Text::new_color(
                                [0.0, 0.0, 0.0, 1.0],
                                (mid_y / 25.).ceil() as u32,
                            )
                            .draw(
                                &format!(
                                    "{:?}",
                                    (((inputnum) / ((10. as f32).powi(itar2 - 4))).round()) * ((10. as f32).powi(itar2 - 4))
                                ),
                                &mut glyph_cache2,
                                &DrawState::default(),
                                c.transform.trans((mid_x * 2.) * (itar / 10.), (mid_y * 2.) - (mid_y / 9.)),
                                gl,
                            )
                            .unwrap();
                            itar += 1.;
                        }
                        text::Text::new_color(
                            [0.0, 0.0, 0.0, 1.0],
                            (mid_y / 25.).ceil() as u32,
                        )
                        .draw(
                            "Zoom",
                            &mut glyph_cache2,
                            &DrawState::default(),
                            c.transform.trans(mid_y / 40., (mid_y * 2.) - (mid_y / 9. - mid_y / 40.)),
                            gl,
                        ).unwrap();
                        text::Text::new_color(
                            [0.0, 0.0, 0.0, 1.0],
                            (mid_y / 25.).ceil() as u32,
                        )
                        .draw(
                            "Visualize MZ Range on Image",
                            &mut glyph_cache2,
                            &DrawState::default(),
                            c.transform.trans(mid_y / 40., (mid_y * 2.) - ((mid_y / 9. - mid_y / 40.) - mid_y / 14.)),
                            gl,
                        ).unwrap();
                        text::Text::new_color(
                            [0.0, 0.0, 0.0, 1.0],
                            (mid_y / 25.).ceil() as u32,
                        )
                        .draw(
                            "Advanced Features",
                            &mut glyph_cache2,
                            &DrawState::default(),
                            c.transform.trans((mid_x * 2.) - (mid_y * 0.533), (mid_y * 2.) - ((mid_y / 9. - mid_y / 55.) - mid_y / 14.)),
                            gl,
                        ).unwrap();
                        text::Text::new_color(
                                [0.0, 0.0, 0.0, 1.0],
                                (mid_y / 20.).ceil() as u32,
                            )
                            .draw(
                                &format!(
                                    "Image MZ min: {:?}, Image MZ max: {:?}",
                                    currentminrange, currentmaxrange
                                ),
                                &mut glyph_cache2,
                                &DrawState::default(),
                                c.transform.trans(mid_x / 8., ((mid_y) + (mid_y / 7.8))),
                                gl,
                            )
                            .unwrap();
                            text::Text::new_color(
                                [0.0, 0.0, 0.0, 1.0],
                                (mid_y / 20.).ceil() as u32,
                            )
                            .draw(
                                &format!(
                                    "Histogram for Spectrum at: {:?}, {:?}, MZ Scaling (Histagram): {:?} to {:?} mz, Scaling: {:?}",
                                    histaspecselection.0, histaspecselection.1, histaminrange, histamaxrange, intensityscale
                                ),
                                &mut glyph_cache2,
                                &DrawState::default(),
                                c.transform.trans(mid_x / 8., ((mid_y) + (mid_y / 20.))),
                                gl,
                            )
                            .unwrap();
                        } else if (stagenum == 0) {
                            text::Text::new_color([0.0, 0.0, 0.0, 1.0], (mid_x / 6.).ceil() as u32)
                                .draw(
                                    "What MS file type are",
                                    &mut glyph_cache,
                                    &DrawState::default(),
                                    c.transform.trans(mid_x - (mid_x * 0.5), (mid_x / 6.)),
                                    gl,
                                )
                                .unwrap();
                            text::Text::new_color([0.0, 0.0, 0.0, 1.0], (mid_x / 6.).ceil() as u32)
                                .draw(
                                    "you trying to access",
                                    &mut glyph_cache,
                                    &DrawState::default(),
                                    c.transform.trans(mid_x - (mid_x * 0.5), (mid_x * 0.28)),
                                    gl,
                                )
                                .unwrap();
                            let rectangle = Rectangle::new(GRAY);
                            let rectangle2 = Rectangle::new(LGRAY);
                            let dims = square(0.0, 0.0, mid_x / 2.);
                            if ((mousex > mid_x - (mid_x / 4. + (mid_x / 3.)))
                                && (mousex < mid_x - ((mid_x / 3.) - mid_x / 4.)))
                                && ((mousey > (mid_y - (mid_x / 4.)))
                                    && (mousey < (mid_y + (mid_x / 4.))))
                            {
                                rectangle2.draw(
                                    dims,
                                    &Default::default(),
                                    c.transform.trans(
                                        mid_x - (mid_x / 4. + (mid_x / 3.)),
                                        mid_y - (mid_x / 4.),
                                    ),
                                    gl,
                                );
                            } else {
                                rectangle.draw(
                                    dims,
                                    &Default::default(),
                                    c.transform.trans(
                                        mid_x - (mid_x / 4. + (mid_x / 3.)),
                                        mid_y - (mid_x / 4.),
                                    ),
                                    gl,
                                );
                            }
                            if ((mousex > mid_x - (mid_x / 4. - (mid_x / 3.)))
                                && (mousex < mid_x + ((mid_x / 3.) + mid_x / 4.)))
                                && ((mousey > (mid_y - (mid_x / 4.)))
                                    && (mousey < (mid_y + (mid_x / 4.))))
                            {
                                rectangle2.draw(
                                    dims,
                                    &Default::default(),
                                    c.transform.trans(
                                        mid_x - (mid_x / 4. - (mid_x / 3.)),
                                        mid_y - (mid_x / 4.),
                                    ),
                                    gl,
                                );
                            } else {
                                rectangle.draw(
                                    dims,
                                    &Default::default(),
                                    c.transform.trans(
                                        mid_x - (mid_x / 4. - (mid_x / 3.)),
                                        mid_y - (mid_x / 4.),
                                    ),
                                    gl,
                                );
                            }
                            text::Text::new_color(
                                [0.0, 0.0, 0.0, 1.0],
                                (mid_x / 12.).ceil() as u32,
                            )
                            .draw(
                                "IBD AND IMZML",
                                &mut glyph_cache,
                                &DrawState::default(),
                                c.transform.trans(
                                    mid_x - (mid_x / 5. + mid_x / 3.),
                                    (mid_y - ((mid_x * 3.) / 24.)),
                                ),
                                gl,
                            )
                            .unwrap();
                            text::Text::new_color(
                                [0.0, 0.0, 0.0, 1.0],
                                (mid_x / 12.).ceil() as u32,
                            )
                            .draw(
                                "OMS Files",
                                &mut glyph_cache,
                                &DrawState::default(),
                                c.transform.trans(
                                    mid_x - (mid_x / 5. - mid_x / 3.),
                                    (mid_y - ((mid_x * 3.) / 24.)),
                                ),
                                gl,
                            )
                            .unwrap();
                            text::Text::new_color(
                                [0.0, 0.0, 0.0, 1.0],
                                (mid_x / 20.).ceil() as u32,
                            )
                            .draw(
                                "Unoptimized and EXTREMELY large",
                                &mut glyph_cache,
                                &DrawState::default(),
                                c.transform.trans(
                                    mid_x - (mid_x / 5. + mid_x / 3.),
                                    (mid_y - ((mid_x * 2.) / 24.)),
                                ),
                                gl,
                            )
                            .unwrap();
                            text::Text::new_color(
                                [0.0, 0.0, 0.0, 1.0],
                                (mid_x / 20.).ceil() as u32,
                            )
                            .draw(
                                "Twenty five times larger then",
                                &mut glyph_cache,
                                &DrawState::default(),
                                c.transform.trans(
                                    mid_x - (mid_x / 5. + mid_x / 3.),
                                    (mid_y - ((mid_x * 1.2) / 24.)),
                                ),
                                gl,
                            )
                            .unwrap();
                            text::Text::new_color(
                                [0.0, 0.0, 0.0, 1.0],
                                (mid_x / 20.).ceil() as u32,
                            )
                            .draw(
                                "OMS format.",
                                &mut glyph_cache,
                                &DrawState::default(),
                                c.transform.trans(
                                    mid_x - (mid_x / 5. + mid_x / 3.),
                                    (mid_y - ((mid_x * 0.4) / 24.)),
                                ),
                                gl,
                            )
                            .unwrap();
                            text::Text::new_color(
                                [0.0, 0.0, 0.0, 1.0],
                                (mid_x / 20.).ceil() as u32,
                            )
                            .draw(
                                "Optimized and quick to load",
                                &mut glyph_cache,
                                &DrawState::default(),
                                c.transform.trans(
                                    mid_x - (mid_x / 5. - mid_x / 3.),
                                    (mid_y - ((mid_x * 2.) / 24.)),
                                ),
                                gl,
                            )
                            .unwrap();
                            text::Text::new_color(
                                [0.0, 0.0, 0.0, 1.0],
                                (mid_x / 20.).ceil() as u32,
                            )
                            .draw(
                                "Accredited to G Newton",
                                &mut glyph_cache,
                                &DrawState::default(),
                                c.transform.trans(
                                    mid_x - (mid_x / 5. - mid_x / 3.),
                                    (mid_y - ((mid_x * 1.2) / 24.)),
                                ),
                                gl,
                            )
                            .unwrap();
                        }
                    });
                }
            }
        }
    }
}
