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
    // 0 to 1
    let mut finishedscreenvisibility = 0.;
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
    let mut pressinginzoombox = false;
    let mut pressinginzoomboxfor: i64 = 1;
    let mut advancedmenu: bool = false;
    let mut textinput: bool = false;
    let mut loadingfinishscreen: bool = false;
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
                            if (stagenum == 1) && advancedmenu {
                            } else if (stagenum == 1) {
                                if (mousey > (mid_y * 2.) - (mid_y / 7.)) {
                                    if (mousex < ((mid_x * 2.) - (mid_y / 7.))) {
                                        if (mousey > (mid_y * 2.) - (mid_y / 14.)) {
                                            pressinginbottombox = true;
                                            currenttempmin = mousex as f32;
                                        } else {
                                            pressinginbottommiddlebox = true;
                                            histatempmin = mousex as f32;
                                        }
                                    } else {
                                        if (mousey < ((mid_y * 2.) - (mid_y / 14.))) {
                                            pressinginzoombox = true;
                                        }
                                    }
                                } else if (mousex > ((mid_x * 2.) - (mid_y / 7.)))
                                    && (mousey > mid_y)
                                    && (mousey < ((mid_y * 2.) - (mid_y / 7.)))
                                {
                                    pressinginsidebox = true;
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
                                    selectedspec = (&mapinmemory)
                                        .arrayspec
                                        .get(
                                            histaspecselection.0 as usize,
                                            histaspecselection.1 as usize,
                                        )
                                        .unwrap()
                                        .to_owned();
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
                                    selectedspec = (&mapinmemory)
                                        .arrayspec
                                        .get(
                                            histaspecselection.0 as usize,
                                            histaspecselection.1 as usize,
                                        )
                                        .unwrap()
                                        .to_owned();
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
                            } else if ((stagenum == 1) && (advancedmenu)) {
                                let ofso = ((mid_x) * ((1. / 8.) + 0.1));
                                if (((mousex > mid_x * 0.2) && (mousex < mid_x * 1.8))
                                    && ((mousey > (mid_x * 0.1))
                                        && (mousey < (mid_x * 0.1) + (mid_x / 8.))))
                                {
                                    let result = nfd::open_save_dialog(Some("oms"), None)
                                        .unwrap_or_else(|e| {
                                            panic!(e);
                                        });
                                    let mut omspath: String = "".to_string();
                                    let mut contin = true;
                                    match result {
                                        Response::Okay(file_path) => omspath = file_path,
                                        Response::OkayMultiple(files) => omspath = files[0].clone(),
                                        Response::Cancel => contin = false,
                                    }
                                    if (contin) {
                                        write_oms_file(
                                            Path::new(&omspath.clone()).to_path_buf(),
                                            &mapinmemory,
                                        );
                                    }
                                }
                                if (((mousex > mid_x * 0.2) && (mousex < mid_x * 1.8))
                                    && ((mousey > (mid_x * 0.1) + (ofso * 1.))
                                        && (mousey < (mid_x * 0.1) + (mid_x / 8.) + (ofso * 1.))))
                                {
                                    let inputtext = gentext(
                                        "List Ranges Formated Like: 'MZmin MZmax, MZmin MZmax,'",
                                        "",
                                        false,
                                        &mut graphics,
                                        &mut window,
                                        &mut events,
                                    );
                                    if !(inputtext.is_none()) {
                                        let result =
                                            nfd::open_pick_folder(None).unwrap_or_else(|e| {
                                                panic!(e);
                                            });
                                        let mut imgdir: String = "".to_string();
                                        let mut contin = true;
                                        match result {
                                            Response::Okay(file_path) => imgdir = file_path,
                                            Response::OkayMultiple(files) => {
                                                imgdir = files[0].clone()
                                            }
                                            Response::Cancel => contin = false,
                                        }
                                        if (contin) {
                                            let intensityarraybool = yesornoquestion(
                                                "Do you want to load txt intensity",
                                                "arrays WITH your images?",
                                                &mut graphics,
                                                &mut window,
                                                &mut events,
                                            );
                                            if (intensityarraybool.is_some()) {
                                                let stringinput = inputtext.unwrap().clone();
                                                let vecs: Vec<&str> =
                                                    stringinput.split(',').collect();
                                                let mut vecsitar = 0.;
                                                let vecslen = vecs.len().clone();
                                                for string in vecs {
                                                    loadingscreen(
                                                        &format!(
                                                            "Generating Ratio Files: {}%",
                                                            (((vecsitar / (vecslen as f32))
                                                                * 10000.)
                                                                .ceil()
                                                                / 100.)
                                                        ),
                                                        &mut graphics,
                                                        &mut window,
                                                        &mut events,
                                                    );
                                                    let mut vecscurrent: Vec<&str> =
                                                        string.split(' ').collect();
                                                    let mut itar = 0;
                                                    while itar < vecscurrent.len() {
                                                        if vecscurrent[itar] == "" {
                                                            vecscurrent.remove(itar);
                                                        } else {
                                                            itar += 1;
                                                        }
                                                    }
                                                    if (vecscurrent.len() == 2)
                                                        && (!(vecscurrent[0]
                                                            .parse::<f64>()
                                                            .is_err()))
                                                        && (!(vecscurrent[1]
                                                            .parse::<f64>()
                                                            .is_err()))
                                                    {
                                                        let img =
                                                            bluegreenimagefromOMSandintensitymap(
                                                                &mapinmemory,
                                                                vecscurrent[0]
                                                                    .parse::<f64>()
                                                                    .unwrap(),
                                                                vecscurrent[1]
                                                                    .parse::<f64>()
                                                                    .unwrap(),
                                                            );
                                                        let mut aoistring = "".to_string();
                                                        img.0.save(Path::new(&imgdir).to_path_buf().join(((("IMG For Range ".to_owned() + &vecscurrent[0]) + " to ") + &vecscurrent[1]) + " (Green used for higher intensity, Blue used for lower, Black used for no signal).png")).unwrap();
                                                        if !intensityarraybool.is_none() {
                                                            if intensityarraybool.unwrap() {
                                                                let mut yval = 0;
                                                                while yval < img.1.num_columns() {
                                                                    let mut xval = 0;
                                                                    while xval < img.1.num_rows() {
                                                                        aoistring = aoistring
                                                                            + &(&img.1)
                                                                                .get(xval, yval)
                                                                                .unwrap()
                                                                                .to_string()
                                                                            + &" ";
                                                                        xval += 1;
                                                                    }
                                                                    aoistring = aoistring + &"\n";
                                                                    yval += 1;
                                                                }
                                                                std::fs::write(Path::new(&imgdir).join(((("Array of intensity for ".to_owned() + vecscurrent[0]) + " to ") + &vecscurrent[1]) + ", Return for every new yval and spaces between each xval.txt"), aoistring).expect("Unable to write file");
                                                            }
                                                        }
                                                    }
                                                    vecsitar += 1.;
                                                }
                                            }
                                            loadingfinishscreen = true;
                                        }
                                    }
                                }
                                if (((mousex > mid_x * 0.2) && (mousex < mid_x * 1.8))
                                    && ((mousey > (mid_x * 0.1) + (ofso * 2.))
                                        && (mousey < (mid_x * 0.1) + (mid_x / 8.) + (ofso * 2.))))
                                {
                                    let inputtext = gentext(
                                        "Formated Like: '[MZmin MZmax, MZmin MZmax], [MZmin...'",
                                        "",
                                        false,
                                        &mut graphics,
                                        &mut window,
                                        &mut events,
                                    );
                                    if !(inputtext.is_none()) {
                                        let result =
                                            nfd::open_pick_folder(None).unwrap_or_else(|e| {
                                                panic!(e);
                                            });
                                        let mut imgdir: String = "".to_string();
                                        let mut contin = true;
                                        match result {
                                            Response::Okay(file_path) => imgdir = file_path,
                                            Response::OkayMultiple(files) => {
                                                imgdir = files[0].clone()
                                            }
                                            Response::Cancel => contin = false,
                                        }
                                        if (contin) {
                                            let mut setsofranges: Vec<((f64, f64), (f64, f64))> =
                                                Vec::new();
                                            let intensityarraybool = yesornoquestion(
                                                "Do you want to load txt versions",
                                                "of your ratiomaps",
                                                &mut graphics,
                                                &mut window,
                                                &mut events,
                                            );
                                            if (intensityarraybool.is_some()) {
                                                let stringinput = inputtext.unwrap().clone();
                                                let vecs: Vec<&str> =
                                                    stringinput.split('[').collect();
                                                let vecslen = vecs.len().clone();
                                                for string in vecs {
                                                    let mut tworanges: Vec<&str> =
                                                        string.split(',').collect();
                                                    
                                                    if (tworanges.len() >= 2) {
                                                        let mut vecscurrent1: Vec<&str> =
                                                            tworanges[0].split(' ').collect();
                                                        let mut vecscurrent2helper: Vec<&str> =
                                                            tworanges[1].split(']').collect();
                                                        let mut vecscurrent2: Vec<&str> =
                                                            vecscurrent2helper[0]
                                                                .split(' ')
                                                                .collect();
                                                        let mut itar = 0;
                                                        while itar < vecscurrent1.len() {
                                                            if vecscurrent1[itar] == "" {
                                                                vecscurrent1.remove(itar);
                                                            } else {
                                                                itar += 1;
                                                            }
                                                        }
                                                        itar = 0;
                                                        while itar < vecscurrent2.len() {
                                                            if vecscurrent2[itar] == "" {
                                                                vecscurrent2.remove(itar);
                                                            } else {
                                                                itar += 1;
                                                            }
                                                        }
                                                        if (vecscurrent1.len() == 2)
                                                            && (vecscurrent2.len() == 2)
                                                            && (!(vecscurrent1[0]
                                                                .parse::<f64>()
                                                                .is_err()))
                                                            && (!(vecscurrent1[1]
                                                                .parse::<f64>()
                                                                .is_err()))
                                                            && (!(vecscurrent2[0]
                                                                .parse::<f64>()
                                                                .is_err()))
                                                            && (!(vecscurrent2[1]
                                                                .parse::<f64>()
                                                                .is_err()))
                                                        {
                                                            setsofranges.push((
                                                                (
                                                                    vecscurrent1[0]
                                                                        .parse::<f64>()
                                                                        .unwrap(),
                                                                    vecscurrent1[1]
                                                                        .parse::<f64>()
                                                                        .unwrap(),
                                                                ),
                                                                (
                                                                    vecscurrent2[0]
                                                                        .parse::<f64>()
                                                                        .unwrap(),
                                                                    vecscurrent2[1]
                                                                        .parse::<f64>()
                                                                        .unwrap(),
                                                                ),
                                                            ))
                                                        }
                                                    }
                                                }
                                            }
                                            circularcorimageoutputfromsetsofmzranges(
                                                &mapinmemory,
                                                setsofranges,
                                                Path::new(&imgdir).to_path_buf(),
                                                intensityarraybool.unwrap(),
                                                &mut graphics,
                                                &mut window,
                                                &mut events,
                                            );
                                            loadingfinishscreen = true;
                                        }
                                    }
                                }
                                if (((mousex > mid_x * 0.2) && (mousex < mid_x * 1.8))
                                    && ((mousey > (mid_x * 0.1) + (ofso * 3.))
                                        && (mousey < (mid_x * 0.1) + (mid_x / 8.) + (ofso * 3.))))
                                {
                                    let inputtext = gentext(
                                        "Function Will Take The Masses Of Sugars An Enzyme Converts From And To",
                                        "Formated Like: '[StartMZmin MZmax, EndMZmin MZmax], [Start MZmin MZmax...",
                                        true,
                                        &mut graphics,
                                        &mut window,
                                        &mut events,
                                    );
                                    if !(inputtext.is_none()) {
                                        let result =
                                            nfd::open_pick_folder(None).unwrap_or_else(|e| {
                                                panic!(e);
                                            });
                                        let mut imgdir: String = "".to_string();
                                        let mut contin = true;
                                        match result {
                                            Response::Okay(file_path) => imgdir = file_path,
                                            Response::OkayMultiple(files) => {
                                                imgdir = files[0].clone()
                                            }
                                            Response::Cancel => contin = false,
                                        }
                                        if (contin) {
                                            let mut setsofranges: Vec<((f64, f64), (f64, f64))> =
                                                Vec::new();

                                                let stringinput = inputtext.unwrap().clone();
                                                let vecs: Vec<&str> =
                                                    stringinput.split('[').collect();
                                                let vecslen = vecs.len().clone();
                                                for string in vecs {
                                                    let mut tworanges: Vec<&str> =
                                                        string.split(',').collect();
                                                    
                                                    if (tworanges.len() >= 2) {
                                                        let mut vecscurrent1: Vec<&str> =
                                                            tworanges[0].split(' ').collect();
                                                        let mut vecscurrent2helper: Vec<&str> =
                                                            tworanges[1].split(']').collect();
                                                        let mut vecscurrent2: Vec<&str> =
                                                            vecscurrent2helper[0]
                                                                .split(' ')
                                                                .collect();
                                                        let mut itar = 0;
                                                        while itar < vecscurrent1.len() {
                                                            if vecscurrent1[itar] == "" {
                                                                vecscurrent1.remove(itar);
                                                            } else {
                                                                itar += 1;
                                                            }
                                                        }
                                                        itar = 0;
                                                        while itar < vecscurrent2.len() {
                                                            if vecscurrent2[itar] == "" {
                                                                vecscurrent2.remove(itar);
                                                            } else {
                                                                itar += 1;
                                                            }
                                                        }
                                                        if (vecscurrent1.len() == 2)
                                                            && (vecscurrent2.len() == 2)
                                                            && (!(vecscurrent1[0]
                                                                .parse::<f64>()
                                                                .is_err()))
                                                            && (!(vecscurrent1[1]
                                                                .parse::<f64>()
                                                                .is_err()))
                                                            && (!(vecscurrent2[0]
                                                                .parse::<f64>()
                                                                .is_err()))
                                                            && (!(vecscurrent2[1]
                                                                .parse::<f64>()
                                                                .is_err()))
                                                        {
                                                            setsofranges.push((
                                                                (
                                                                    vecscurrent1[0]
                                                                        .parse::<f64>()
                                                                        .unwrap(),
                                                                    vecscurrent1[1]
                                                                        .parse::<f64>()
                                                                        .unwrap(),
                                                                ),
                                                                (
                                                                    vecscurrent2[0]
                                                                        .parse::<f64>()
                                                                        .unwrap(),
                                                                    vecscurrent2[1]
                                                                        .parse::<f64>()
                                                                        .unwrap(),
                                                                ),
                                                            ))
                                                        }
                                                    }
                                                }
                    
                                            genenzymepresenceimgs(
                                                &mapinmemory,
                                                setsofranges,
                                                &mut graphics,
                                                &mut window,
                                                &mut events,
                                            );
                                            loadingfinishscreen = true;
                                        }
                                    }
                                }
                                if (((mousex > mid_x * 0.2) && (mousex < mid_x * 1.8))
                                    && ((mousey > (mid_x * 0.1) + (ofso * 4.))
                                        && (mousey < (mid_x * 0.1) + (mid_x / 8.) + (ofso * 4.))))
                                {
                                    advancedmenu = false;
                                }
                            } else if (stagenum == 1) {
                                if (pressinginzoombox) {
                                    pressinginzoomboxfor = 0;
                                    pressinginzoombox = false;
                                }
                                if (mousex > (mid_x * 2.) - (mid_y / 7.))
                                    && (mousey > (mid_y * 2.) - (mid_y / 14.))
                                {
                                    pressinginbottombox = false;
                                    pressinginbottommiddlebox = false;
                                    pressinginsidebox = false;
                                    pressinginzoombox = false;
                                    advancedmenu = true;
                                }
                                if (pressinginsidebox) {
                                    let mut intensityscaletemp = (((mid_y * 2.) - mousey)
                                        - (mid_y / 7.))
                                        / (mid_y - (mid_y / 7.));
                                    if (intensityscaletemp > 1.) {
                                        intensityscaletemp = 1.;
                                    }
                                    if (intensityscaletemp < 0.) {
                                        intensityscaletemp = 0.;
                                    }
                                    intensityscaletemp = intensityscaletemp * 2.;
                                    if (intensityscaletemp > 1.) {
                                        intensityscaletemp = 1. / (2. - intensityscaletemp);
                                    }
                                    intensityscale = intensityscale
                                        * (1. / (intensityscaletemp + 0.001) + 0.001);
                                    pressinginsidebox = false;
                                }
                                if (pressinginbottommiddlebox) {
                                    let histaminrange2 = histaminrange
                                        + (((histatempmin) / ((mid_x * 2.) as f32))
                                            * (histamaxrange - histaminrange));
                                    let histamaxrange2 = histaminrange
                                        + (((mousex as f32) / ((mid_x * 2.) as f32))
                                            * (histamaxrange - histaminrange));
                                    if (histaminrange2 > histamaxrange2) {
                                        histamaxrange = histaminrange2.clone() + 0.02;
                                        histaminrange = histamaxrange2.clone();
                                    } else {
                                        histaminrange = histaminrange2.clone();
                                        histamaxrange = histamaxrange2.clone() + 0.02;
                                    }
                                    pressinginbottommiddlebox = false;
                                }
                                if (pressinginbottombox) {
                                    let histaminrange2 = histaminrange
                                        + (((currenttempmin as f32) / ((mid_x * 2.) as f32))
                                            * (histamaxrange - histaminrange));
                                    let histamaxrange2 = histaminrange
                                        + (((mousex as f32) / ((mid_x * 2.) as f32))
                                            * (histamaxrange - histaminrange));
                                    if (histaminrange2 > histamaxrange2) {
                                        currentminrange = histamaxrange2.clone() as f64;
                                        currentmaxrange = histaminrange2.clone() as f64;
                                    } else {
                                        currentminrange = histaminrange2.clone() as f64;
                                        currentmaxrange = histamaxrange2.clone() as f64;
                                    }
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
                                    pressinginbottombox = false;
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
                                    [0.33, 0., 0., 1.],
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
                                        intensityscale as f32,
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
                                        intensityscale as f32,
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
                                        intensityscale as f32,
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
                                        intensityscale as f32,
                                    ),
                                    c.transform,
                                    gl,
                                );
                            }
                            let rectangle4 = Rectangle::new([1.0, 0.66, 0.66, 1.0]);
                            let rectangle3 = Rectangle::new(WHITE);
                            let rectangle = Rectangle::new(BLACK);
                            rectangle3.draw(
                                [0., 0., mid_y / 7., mid_y * 2.],
                                &Default::default(),
                                c.transform.trans((mid_x * 2.) - (mid_y / 7.), 0.),
                                gl,
                            );
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
                                [mid_x * 2. - (mid_y / 7.), (mid_y) + (mid_y / 15.)],
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
                            if (pressinginsidebox) {
                                let mut mousey2 = 0.;
                                if (mousey > mid_y){
                                    if(mousey < (mid_y * 2.) - (mid_y / 7.)){
                                        mousey2 = mousey;
                                    }else{
                                        mousey2 = (mid_y * 2.) - (mid_y / 7.);
                                    }
                                }else{
                                    mousey2 = mid_y;
                                }
                                let rectangle5 = Rectangle::new([((mousey2 - (mid_y / 7.)) / ((mid_y * 2.) - (mid_y / 7.))) as f32, 0.66, 0.66, 1.0]);
                                rectangle5.draw(
                                    [
                                        0.,
                                        0.,
                                        mid_y / 7.,
                                        ((mid_y * 2.) - mousey2) - (((mid_y - (mid_y / 7.)) / 2.) + (mid_y / 7.)),
                                    ],
                                    &Default::default(),
                                    c.transform.trans(
                                        (mid_x * 2.) - (mid_y / 7.) + 2.,
                                        mousey2,
                                    ),
                                    gl,
                                );
                            }
                            line_from_to(
                                BLACK,
                                1.0,
                                [mid_x * 2., ((mid_y * 2.) - ((mid_y - (mid_y / 7.)) / 2. + (mid_y / 7.)))],
                                [mid_x * 2. - (mid_y / 7.), ((mid_y * 2.) - ((mid_y - (mid_y / 7.)) / 2. + (mid_y / 7.)))],
                                c.transform,
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
                            if (pressinginbottombox) {
                                let mut offs = 0.;
                                if (currenttempmin > mousex as f32){
                                    offs = mousex;
                                }else{
                                    offs = currenttempmin as f64;
                                }
                                rectangle4.draw(
                                    [
                                        0.,
                                        0.,
                                        (mousex - currenttempmin as f64).abs(),
                                        mid_y / 14.,
                                    ],
                                    &Default::default(),
                                    c.transform.trans(
                                        offs,
                                        (mid_y * 2.) - mid_y / 14.,
                                    ),
                                    gl,
                                );
                            }
                            line_from_to(
                                BLACK,
                                2.0,
                                [0., (mid_y * 2.) - (mid_y / 7.)],
                                [mid_x * 2., (mid_y * 2.) - (mid_y / 7.)],
                                c.transform,
                                gl,
                            );
                            let rectangle6 = Rectangle::new([0.66, 0.66, 0.66, 1.0]);
                            let rectangle6light = Rectangle::new([0.8, 0.8, 0.8, 1.0]);
                            let rectangle6lightred = Rectangle::new([1., 0.7, 0.7, 1.0]);
                            let mut itar: f64 = 0.5;
                            while itar < 8. {
                            let inputnum = histaminrange + ((histamaxrange - histaminrange) * ((itar / 10.) as f32));
                            let mut inputnumrep = inputnum.clone();
                            let mut itar2 = 0;
                            while (inputnumrep >= 0.000001){
                                inputnumrep *= 0.1;
                                itar2 += 1;
                            }
                            itar2 -= 6;
                            line_from_to(
                                BLACK,
                                1.0,
                                [(mid_x * 2.) * ((itar) / 10.), (mid_y * 2.) - (mid_y / 7.)],
                                [(mid_x * 2.) * ((itar) / 10.), (mid_y * 2.) - (mid_y / 7.) + (mid_y / 50.)],
                                c.transform,
                                gl,
                            );
                            text::Text::new_color(
                                [0.0, 0.0, 0.0, 1.0],
                                (mid_y / 25.).ceil() as u32,
                            )
                            .draw(
                                &format!(
                                    "{:?}",
                                    (((inputnum) / ((10. as f32).powi(itar2 - 7))).round()) * ((10. as f32).powi(itar2 - 7))
                                ),
                                &mut glyph_cache2,
                                &DrawState::default(),
                                c.transform.trans((mid_x * 2.) * ((itar - 0.25) / 10.), (mid_y * 2.) - (mid_y / 9.) + (mid_y / 50.)),
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
                            c.transform.trans(mid_y / 40., (mid_y * 2.) - (mid_y / 9. - mid_y / 40.) - (mid_y / 14.)),
                            gl,
                        ).unwrap();
                        text::Text::new_color(
                            [0.0, 0.0, 0.0, 1.0],
                            (mid_y / 25.).ceil() as u32,
                        )
                        .draw(
                            "Scale",
                            &mut glyph_cache2,
                            &DrawState::default(),
                            c.transform.trans(mid_x * 2. - (mid_y / 8.), mid_y + (mid_y / 20.)),
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
                            (mid_y / 25.).ceil() as u32,
                        )
                        .draw(
                            "Zoom Out",
                            &mut glyph_cache2,
                            &DrawState::default(),
                            c.transform.trans((mid_x * 2.) - (mid_y * 0.35), (mid_y * 2.) - ((mid_y / 9. - mid_y / 55.))),
                            gl,
                        ).unwrap();
                        if (pressinginzoombox && !advancedmenu) {
                            pressinginzoomboxfor += 1;
                            let range = histamaxrange - histaminrange;
                            histaminrange = histaminrange - (((100. + (pressinginzoomboxfor as f32).powf(0.3)) / 10000.) * range);
                            histamaxrange = histamaxrange + (((100. + (pressinginzoomboxfor as f32).powf(0.3)) / 10000.) * range);
                            rectangle6lightred.draw(
                                [
                                    0.,
                                    0.,
                                    mid_y / 7.,
                                    mid_y / 14. - 2.,
                                ],
                                &Default::default(),
                                c.transform.trans(
                                    mid_x * 2. - (mid_y / 7.) + 2.,
                                    mid_y * 2. - (mid_y / 7.) + 2.,
                                ),
                                gl,
                            );
                        } else if (!advancedmenu) && (mousex > (mid_x * 2.) - (mid_y / 7.)) && (mousey > (mid_y * 2.) - (mid_y / 7.)) && (mousey < (mid_y * 2.) - (mid_y / 14.)) {
                            rectangle6light.draw(
                                [
                                    0.,
                                    0.,
                                    mid_y / 7.,
                                    mid_y / 14. - 2.,
                                ],
                                &Default::default(),
                                c.transform.trans(
                                    mid_x * 2. - (mid_y / 7.) + 2.,
                                    mid_y * 2. - (mid_y / 7.) + 2.,
                                ),
                                gl,
                            );
                        }else{
                            rectangle6.draw(
                                [
                                    0.,
                                    0.,
                                    mid_y / 7.,
                                    mid_y / 14. - 2.,
                                ],
                                &Default::default(),
                                c.transform.trans(
                                    mid_x * 2. - (mid_y / 7.),
                                    mid_y * 2. - (mid_y / 7.) + 2.,
                                ),
                                gl,
                            );
                        }
                        if (!advancedmenu) && (mousex > (mid_x * 2.) - (mid_y / 7.)) && (mousey > (mid_y * 2.) - (mid_y / 14.)) {
                            rectangle6light.draw(
                                [
                                    0.,
                                    0.,
                                    mid_y / 7.,
                                    mid_y / 14. - 1.,
                                ],
                                &Default::default(),
                                c.transform.trans(
                                    mid_x * 2. - (mid_y / 7.) + 2.,
                                    mid_y * 2. - (mid_y / 14.) + 1.,
                                ),
                                gl,
                            );
                        }else{
                            rectangle6.draw(
                                [
                                    0.,
                                    0.,
                                    mid_y / 7.,
                                    mid_y / 14. - 1.,
                                ],
                                &Default::default(),
                                c.transform.trans(
                                    mid_x * 2. - (mid_y / 7.) + 2.,
                                    mid_y * 2. - (mid_y / 14.) + 1.,
                                ),
                                gl,
                            );
                        }
                        line_from_to(
                            BLACK,
                            2.0,
                            [(mid_x * 2.) - (mid_y / 7.), mid_y],
                            [(mid_x * 2.) - (mid_y / 7.), (mid_y * 2.)],
                            c.transform,
                            gl,
                        );
                        line_from_to(
                            BLACK,
                            1.0,
                            [(mid_x * 2.) - (mid_y / 7.), (mid_y * 2.) - (mid_y / 14.)],
                            [(mid_x * 2.), (mid_y * 2.) - (mid_y / 14.)],
                            c.transform,
                            gl,
                        );
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
                            if (advancedmenu){
                                let rectangle7 = Rectangle::new([0., 0., 0., 0.93]);
                                let rectangle8 = Rectangle::new_round(GRAY, mid_x / 16.);
                                let rectangle8l = Rectangle::new_round(LGRAY, mid_x / 16.);
                                rectangle7.draw(
                                    [
                                        0.,
                                        0.,
                                        mid_x * 2.,
                                        mid_y * 2.,
                                    ],
                                    &Default::default(),
                                    c.transform.trans(
                                        0.,
                                        0.,
                                    ),
                                    gl,
                                );
                                let mut itarr = 0;
                                let ofso = (((mid_x) * ((1. / 8.) + 0.1)));
                                while itarr < 5 {
                                    let ofs = (itarr as f64 * ofso);
                                    if (((mousex > mid_x * 0.2) && (mousex < mid_x * 1.8)) && ((mousey > (mid_x * 0.1) + ofs) && (mousey < (mid_x * 0.1) + (mid_x / 8.) + ofs))) {
                                        rectangle8l.draw(
                                            [
                                                0.,
                                                0.,
                                                mid_x * 1.6,
                                                mid_x / 8.,
                                            ],
                                            &Default::default(),
                                            c.transform.trans(
                                                mid_x * 0.2,
                                                mid_x * 0.1 + ofs,
                                            ),
                                            gl,
                                        );
                                    }else{
                                        rectangle8.draw(
                                            [
                                                0.,
                                                0.,
                                                mid_x * 1.6,
                                                mid_x / 8.,
                                            ],
                                            &Default::default(),
                                            c.transform.trans(
                                                mid_x * 0.2,
                                                mid_x * 0.1 + ofs,
                                            ),
                                            gl,
                                        );
                                    }
                                    itarr += 1;
                                }
                                text::Text::new_color([0.0, 0.0, 0.0, 1.0], (mid_x / 16.).ceil() as u32)
                                .draw(
                                    "Download As OMS (Faster / Smaller Data Type)",
                                    &mut glyph_cache2,
                                    &DrawState::default(),
                                    c.transform.trans(mid_x * 0.25, ((mid_x * 0.1) + (((mid_x / 8.) - (mid_x / 16.)) / 2.) + (mid_x / 30.))), gl,
                                )
                                .unwrap();
                                text::Text::new_color([0.0, 0.0, 0.0, 1.0], (mid_x / 35.).ceil() as u32)
                                .draw(
                                    "Compressed, Incredibly Quick To Load, Not Great For Machines With Not-so Steep Slopes",
                                    &mut glyph_cache2,
                                    &DrawState::default(),
                                    c.transform.trans(mid_x * 0.25, ((mid_x * 0.1) + (((mid_x / 8.) - (mid_x / 16.)) / 2.) + (mid_x / 15.))), gl,
                                )
                                .unwrap();
                                text::Text::new_color([0.0, 0.0, 0.0, 1.0], (mid_x / 16.).ceil() as u32)
                                .draw(
                                    "Generate Images From MZ Range",
                                    &mut glyph_cache2,
                                    &DrawState::default(),
                                    c.transform.trans(mid_x * 0.25, (ofso * 1.) + ((mid_x * 0.1) + (((mid_x / 8.) - (mid_x / 16.)) / 2.) + (mid_x / 30.))), gl,
                                )
                                .unwrap();
                                text::Text::new_color([0.0, 0.0, 0.0, 1.0], (mid_x / 35.).ceil() as u32)
                                .draw(
                                    "Displays The Signal For Each Spectra (W/ .PNG) Given An MZ Range",
                                    &mut glyph_cache2,
                                    &DrawState::default(),
                                    c.transform.trans(mid_x * 0.25, (ofso * 1.) + ((mid_x * 0.1) + (((mid_x / 8.) - (mid_x / 16.)) / 2.) + (mid_x / 15.))), gl,
                                )
                                .unwrap();
                                text::Text::new_color([0.0, 0.0, 0.0, 1.0], (mid_x / 16.).ceil() as u32)
                                .draw(
                                    "Generate Ratios/Coril Maps <- (MZ Range, MZ Range)",
                                    &mut glyph_cache2,
                                    &DrawState::default(),
                                    c.transform.trans(mid_x * 0.25, (ofso * 2.) + ((mid_x * 0.1) + (((mid_x / 8.) - (mid_x / 16.)) / 2.) + (mid_x / 30.))), gl,
                                )
                                .unwrap();
                                text::Text::new_color([0.0, 0.0, 0.0, 1.0], (mid_x / 35.).ceil() as u32)
                                .draw(
                                    "Displays The Ratio Of Two MZ Ranges For Each Spectra Using Blue for the First Range And Green For The Second",
                                    &mut glyph_cache2,
                                    &DrawState::default(),
                                    c.transform.trans(mid_x * 0.25, (ofso * 2.) + ((mid_x * 0.1) + (((mid_x / 8.) - (mid_x / 16.)) / 2.) + (mid_x / 15.))), gl,
                                )
                                .unwrap();
                                text::Text::new_color([0.0, 0.0, 0.0, 1.0], (mid_x / 16.).ceil() as u32)
                                .draw(
                                    "Find Enzyme Presence From (MZ Range, MZ Range)",
                                    &mut glyph_cache2,
                                    &DrawState::default(),
                                    c.transform.trans(mid_x * 0.25, (ofso * 3.) + ((mid_x * 0.1) + (((mid_x / 8.) - (mid_x / 16.)) / 2.) + (mid_x / 30.))), gl,
                                )
                                .unwrap();
                                text::Text::new_color([0.0, 0.0, 0.0, 1.0], (mid_x / 35.).ceil() as u32)
                                .draw(
                                    "Complex Algorithm To Find Enzymes Within A Sample, Input (MZ For Start, MZ For Result) Per Enzyme",
                                    &mut glyph_cache2,
                                    &DrawState::default(),
                                    c.transform.trans(mid_x * 0.25, (ofso * 3.) + ((mid_x * 0.1) + (((mid_x / 8.) - (mid_x / 16.)) / 2.) + (mid_x / 15.))), gl,
                                )
                                .unwrap();
                                text::Text::new_color([0.0, 0.0, 0.0, 1.0], (mid_x / 16.).ceil() as u32)
                                .draw(
                                    "Exit Advanced Menu",
                                    &mut glyph_cache2,
                                    &DrawState::default(),
                                    c.transform.trans(mid_x * 0.25, (ofso * 4.) + ((mid_x * 0.1) + (((mid_x / 8.) - (mid_x / 16.)) / 2.) + (mid_x / 30.))), gl,
                                )
                                .unwrap();
                                text::Text::new_color([0.0, 0.0, 0.0, 1.0], (mid_x / 35.).ceil() as u32)
                                .draw(
                                    "Exit Out Of The Advanced Options Menu, Doesn't Exit The Program.",
                                    &mut glyph_cache2,
                                    &DrawState::default(),
                                    c.transform.trans(mid_x * 0.25, (ofso * 4.) + ((mid_x * 0.1) + (((mid_x / 8.) - (mid_x / 16.)) / 2.) + (mid_x / 15.))), gl,
                                )
                                .unwrap();
                            }
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
                        if(loadingfinishscreen){
                            if(finishedscreenvisibility > 0.97){
                                finishedscreenvisibility = 1.;
                                loadingfinishscreen = false;
                            }else{
                                finishedscreenvisibility += 0.03;
                            }
                        }else{
                            if (finishedscreenvisibility - 0.0075 > 0.) {
                                finishedscreenvisibility -= 0.0075;
                            }else{
                                finishedscreenvisibility = 0.;
                            }
                        }
                        if (finishedscreenvisibility > 0.) {
                            let rectangle = Rectangle::new_round([1., 1., 1., finishedscreenvisibility * 0.85], mid_x / 14.);
                            let rectangle2 = Rectangle::new([0., 0., 0., finishedscreenvisibility * 0.425]);
                                rectangle2.draw(
                                     [
                                         0.,
                                         0.,
                                        mid_x * 2.,
                                        mid_y * 2.,
                                    ],
                                    &Default::default(),
                                    c.transform.trans(
                                        0.,
                                        0.,
                                    ),
                                    gl,
                                );
                            rectangle.draw(
                                [0., 0., (mid_x * 2.) / 3., (mid_x * 2.) / 5.],
                                &Default::default(),
                                c.transform.trans((mid_x * 2.) / 3., ((mid_y * 2.) - (mid_x * 2.) / 5.) / 2.),
                                gl,
                            );
                            text::Text::new_color(
                                [0.0, 0.0, 0.0, finishedscreenvisibility * 0.85],
                                (mid_x / 8.).ceil() as u32,
                            )
                            .draw(
                                "Finished!",
                                &mut glyph_cache2,
                                &DrawState::default(),
                                c.transform.trans(
                                    (mid_x * 2.) / 3. + (mid_x / 15.),
                                    ((mid_y) + (mid_x / 5.)) - ((((mid_x * 2.) / 5.) - (mid_x / 10.)) / 2.),
                                ),
                                gl,
                            ).unwrap();
                        }
                    });
                }
            }
        }
    }
}
