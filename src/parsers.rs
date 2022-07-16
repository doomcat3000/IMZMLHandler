use crate::imzml_types::*;
use ::image::io::Reader;
use ::image::DynamicImage;
use ::image::{GenericImage, GenericImageView, ImageBuffer, Pixel, Pixels, RgbImage};
use array2d::*;
use byteorder::{BigEndian, LittleEndian, ReadBytesExt, WriteBytesExt};
use glutin_window::GlutinWindow as Window;
use graphics::rectangle::square;
use graphics::Image;
use graphics::*;
use minidom::NSChoice;
use nfd::Response;
use opengl_graphics::*;
use opengl_graphics::{GlGraphics, OpenGL};
use piston::event_loop::{EventSettings, Events};
use piston::input::{ButtonEvent, Event, RenderEvent};
use piston::window::WindowSettings;
use piston::Button::Keyboard;
use piston::ButtonState::Release;
use piston::Key;
use std::fs::File;
use std::io::Cursor;
use std::io::Read;
use std::path::Path;
use std::path::PathBuf;
use std::str::*;
use std::time::Instant;
use std::*;
pub fn loadingscreen(
    text: &str,
    mut gl: &mut GlGraphics,
    window: &mut Window,
    events: &mut Events,
) {
    const WHITE: [f32; 4] = [1.0, 1.0, 1.0, 1.0];
    const BLACK: [f32; 4] = [0.0, 0.0, 0.0, 1.0];
    let mut glyph_cache = GlyphCache::new(
        "assets/PlayfairDisplayRegular-ywLOY.ttf",
        (),
        TextureSettings::new(),
    )
    .unwrap();
    if let Some(evt) = events.next(window) {
        if let Some(args) = evt.render_args() {
            clear(WHITE, gl);
            (&mut gl).draw(args.viewport(), |c, gl2| {
                let (mid_x, mid_y) = (args.window_size[0] / 2.0, args.window_size[1] / 2.0);
                text::Text::new_color([0.0, 0.0, 0.0, 1.0], (mid_x / 10.).ceil() as u32)
                    .draw(
                        text,
                        &mut glyph_cache,
                        &DrawState::default(),
                        c.transform.trans((mid_x / 6.), (mid_x / 6.)),
                        gl2,
                    )
                    .unwrap();
            })
        }
    }
}
pub fn get_file_as_byte_vec(filepath: PathBuf) -> Vec<u8> {
    let mut f = File::open(&filepath).expect("no file found");
    let metadata = fs::metadata(&filepath).expect("unable to read metadata");
    let mut buffer = vec![0; metadata.len() as usize];
    f.read(&mut buffer).expect("buffer overflow");

    buffer
}
pub fn write_oms_file(filepath: PathBuf, oms: OptimizedMSMap) {
    let mut xval = 0;
    let mut arrayspecvec = Vec::new();
    while xval < oms.arrayspec.num_rows() {
        let mut yval = 0;
        while yval < oms.arrayspec.num_columns() {
            arrayspecvec.push(oms.arrayspec.get(xval, yval).unwrap().clone());
            yval += 1;
        }
        xval += 1;
    }
    let mut omst = OptimizedMSMapTransitionstate {
        arrayspec: arrayspecvec,
        sizex: oms.arrayspec.num_rows(),
        sizey: oms.arrayspec.num_columns(),
    };
    let omsbytes: Vec<u8> = bincode::serialize(&omst).unwrap();
    fs::write(filepath, omsbytes).unwrap();
}
pub fn read_oms_file(filepath: PathBuf) -> OptimizedMSMap {
    let omsbytes: Vec<u8> = get_file_as_byte_vec(filepath);
    let omst: OptimizedMSMapTransitionstate = bincode::deserialize(&omsbytes).unwrap();
    let mut oms: OptimizedMSMap = OptimizedMSMap::new(omst.sizex, omst.sizey);
    let mut index = 0;
    let mut xval = 0;
    while xval < oms.arrayspec.num_rows() {
        let mut yval = 0;
        while yval < oms.arrayspec.num_columns() {
            oms.arrayspec.set(xval, yval, omst.arrayspec[index].clone());
            yval += 1;
            index += 1;
        }
        xval += 1;
    }
    return oms;
}
pub fn getslope(mzpoint1: Mzplusintensity, mzpoint2: Mzplusintensity) -> f64 {
    (mzpoint2.intensity - mzpoint1.intensity) as f64 / (mzpoint2.mz - mzpoint1.mz)
}
pub fn getpeakavrslope(index: usize, spec: &Spectrum) -> f64 {
    let mut scanpos: bool = true;
    let mut scanneg: bool = true;
    let mut inde = 1;
    let mut indeneg: usize = 1;
    let mut slopevec: Vec<f64> = Vec::new();
    while scanpos {
        if (spec.points.len() > inde + index) {
            if (spec.points[index + inde].intensity < spec.points[index + inde - 1].intensity) {
                inde = inde + 1;
            } else {
                slopevec.push(getslope(spec.points[index + inde], spec.points[index]));
                scanpos = false;
            }
        } else {
            if (inde > 1) {
                slopevec.push(getslope(spec.points[index + inde - 1], spec.points[index]));
            }
            scanpos = false;
        }
    }
    while scanneg {
        if (0 < index as i64 - indeneg as i64) {
            if (spec.points[index - indeneg].intensity < spec.points[index - indeneg + 1].intensity)
            {
                indeneg = indeneg + 1;
            } else {
                slopevec.push(getslope(spec.points[index - indeneg], spec.points[index]));
                scanneg = false;
            }
        } else {
            if (indeneg > 1) {
                slopevec.push(getslope(spec.points[index + inde - 1], spec.points[index]));
            }
            scanneg = false;
        }
    }
    let mut slopepermz = 10000.;
    let mut num = 0.;
    for slope in &slopevec {
        num = slope.abs() + num;
    }
    if num > 0. {
        num = num / slopevec.len() as f64;
        slopepermz = num;
    }
    return slopepermz;
}
pub fn optimizespectrum(spec: Spectrum) -> OSpectrum {
    let mut peaklist: Vec<USpecPeeks> = Vec::new();
    let mut pointindex: i64 = 0;
    let mut maxintensity: f32 = 0.;
    let mut minmz: f32 = 0.;
    let mut maxmz: f32 = 0.;
    for mzinpoint in &spec.points {
        if pointindex == 0 {
            if mzinpoint.intensity > spec.points[(pointindex as usize) + 1].intensity {
                peaklist.push(USpecPeeks::new(
                    mzinpoint.mz,
                    mzinpoint.intensity,
                    getpeakavrslope(pointindex as usize, &spec),
                ));
            }
        } else {
            if pointindex < (&spec.points.len() - 1) as i64 {
                if (mzinpoint.intensity > spec.points[(pointindex as usize) - 1].intensity)
                    && (mzinpoint.intensity > spec.points[(pointindex as usize) + 1].intensity)
                {
                    peaklist.push(USpecPeeks::new(
                        mzinpoint.mz,
                        mzinpoint.intensity,
                        getpeakavrslope(pointindex as usize, &spec),
                    ));
                }
            } else {
                if mzinpoint.intensity > spec.points[(pointindex as usize) - 1].intensity {
                    peaklist.push(USpecPeeks::new(
                        mzinpoint.mz,
                        mzinpoint.intensity,
                        getpeakavrslope(pointindex as usize, &spec),
                    ));
                }
            }
        }
        if (mzinpoint.mz as f32) > maxmz {
            maxmz = mzinpoint.mz as f32;
        }
        if (mzinpoint.mz as f32) < minmz {
            minmz = mzinpoint.mz as f32;
        }
        if mzinpoint.intensity > maxintensity {
            maxintensity = mzinpoint.intensity;
        }
        pointindex = pointindex + 1;
    }
    let mut i = 0;
    let mut optimizedpeaklist: Vec<OSpecPeeks> = Vec::new();
    for mzin in peaklist {
        optimizedpeaklist.push(OSpecPeeks::new(
            ((mzin.mz as f32 - minmz) / (maxmz - minmz) * 4294967295.).ceil() as u32,
            ((mzin.intensity) / (maxintensity) * 65535.).ceil() as u16,
            (mzin.slopepermz / 10.).ceil() as u16,
        ));
    }
    return OSpectrum::new(maxintensity, optimizedpeaklist, minmz, maxmz);
}
pub fn readoptimizedMSmap(
    ibdpath: PathBuf,
    imzmlpath: PathBuf,
    gl: &mut GlGraphics,
    window: &mut Window,
    events: &mut Events,
) -> OptimizedMSMap {
    let inputunoptimmapms: UnoptimizedMSMap =
        unoptimizedparseimzml(ibdpath, imzmlpath, gl, window, events);
    return UMSmaptoOMSmap(inputunoptimmapms);
}
pub fn UMSmaptoOMSmap(map: UnoptimizedMSMap) -> OptimizedMSMap {
    let mut newomsmap: OptimizedMSMap = OptimizedMSMap::new(map.xsize as usize, map.ysize as usize);
    let mut i: i64 = 0;
    let rootspeclen: usize = map.vecspectrum.len();
    for spec in map.vecspectrum {
        if (i as f64 / 1000.).ceil() == i as f64 / 1000. {
            println!(
                "Optimizing {:?}% Complete",
                ((i as f64 * 10000.) / (rootspeclen as f64)).ceil() / 100.
            );
        }
        newomsmap.arrayspec.set(
            spec.x as usize - 1,
            spec.y as usize - 1,
            optimizespectrum(spec),
        );
        i = i + 1;
    }
    newomsmap
}
pub fn OSpecPeeksToUSpecPeeks(optimizedpeek: OSpecPeeks) -> USpecPeeks {
    todo!();
}
// Margin of error should be less than 0.2
pub fn bluegreenimagefromOMS(omsmap: &OptimizedMSMap, minrange: f64, maxrange: f64) -> RgbImage {
    let arrayofintensityandh: (Array2D<f64>, f64) = intensitymapfromOMS(omsmap, minrange, maxrange);
    let arrayofintensity = arrayofintensityandh.0;
    let highestsignal = arrayofintensityandh.1;
    let mut img: RgbImage = DynamicImage::new_rgb8(
        omsmap.arrayspec.num_rows() as u32,
        omsmap.arrayspec.num_columns() as u32,
    )
    .to_rgb8();
    let mut xval = 0;
    while xval < omsmap.arrayspec.num_rows() {
        let mut yval = 0;
        while yval < omsmap.arrayspec.num_columns() {
            let intensitypercentage: f64 =
                (arrayofintensity.get(xval, yval).unwrap() / highestsignal);
            let mut red: u8 = 0;
            let mut blue: u8 = 0;
            let mut green: u8 = 0;
            let intensitymappedas32bit: f64 = (intensitypercentage * 1020.);
            if (intensitymappedas32bit <= 255.) {
                blue = intensitymappedas32bit.round() as u8;
            } else {
                if (intensitymappedas32bit < 765.) {
                    blue = (255. - ((intensitymappedas32bit - 255.) / 2.)).round() as u8;
                    green = ((intensitymappedas32bit - 255.) / 2.).round() as u8;
                } else {
                    green = 255;
                }
            }
            img.put_pixel(xval as u32, yval as u32, ::image::Rgb([red, green, blue]));
            yval = yval + 1;
        }
        xval = xval + 1;
    }
    img
}
pub fn redimagefromOMS(omsmap: &OptimizedMSMap, minrange: f64, maxrange: f64) -> RgbImage {
    let arrayofintensityandh: (Array2D<f64>, f64) = intensitymapfromOMS(omsmap, minrange, maxrange);
    let arrayofintensity = arrayofintensityandh.0;
    let highestsignal = arrayofintensityandh.1;
    let mut img: RgbImage = DynamicImage::new_rgb8(
        omsmap.arrayspec.num_rows() as u32,
        omsmap.arrayspec.num_columns() as u32,
    )
    .to_rgb8();
    let mut xval = 0;
    while xval < omsmap.arrayspec.num_rows() {
        let mut yval = 0;
        while yval < omsmap.arrayspec.num_columns() {
            img.put_pixel(
                xval as u32,
                yval as u32,
                ::image::Rgb([
                    ((arrayofintensity.get(xval, yval).unwrap() / highestsignal) * 255.).ceil()
                        as u8,
                    0,
                    0,
                ]),
            );
            yval = yval + 1;
        }
        xval = xval + 1;
    }
    img
}
pub fn peekeval(spec: &OSpectrum, peek: &OSpecPeeks, minrange: f64, maxrange: f64) -> f64 {
    let mut out: f64 = 0.;
    let peekintensity =
        ((peek.intensityrelativetomax as f64 / 65535.) * spec.highestintensity as f64);
    let peekslope = (peek.slopepermzover10 as f64 * 10.);
    let peekms = ((spec.max - spec.min) as f64 * (peek.relativemztominandmax as f64 / 4294967295.))
        + spec.min as f64;
    let peekwidth = (peekintensity / peekslope);
    if (peekms > minrange - peekwidth) && (peekms < maxrange + peekwidth) {
        if (peekms > minrange) && (peekms < maxrange) {
            if (peekms > minrange + peekwidth) && (peekms < maxrange - peekwidth) {
                out = out + peekwidth.powi(2) * peekslope;
            } else {
                if (peekms < minrange + peekwidth) {
                    out = out + peekwidth.powi(2) * peekslope
                        - ((minrange + peekwidth - peekms).powi(2) * peekslope / 2.);
                } else {
                    if (peekms > maxrange - peekwidth) {
                        out = out + peekwidth.powi(2) * peekslope
                            - ((peekms - (maxrange - peekwidth)).powi(2) * peekslope / 2.);
                    }
                }
            }
        } else {
            if (peekms < minrange) {
                out = out + (peekms - (minrange - peekwidth)).powi(2) * peekslope / 2.;
            } else {
                out = out + ((maxrange + peekwidth) - peekms).powi(2) * peekslope / 2.;
            }
        }
    }
    out
}
pub fn multibluegreenimagefromOMS(
    omsmapf: OptimizedMSMap,
    ranges: Vec<(f64, f64)>,
) -> Vec<RangeAndRGBImage> {
    let arrayofintensityandh: (Vec<RangeAndIntensityMap>, OptimizedMSMap) =
        multiintensitymapsfromOMS(omsmapf, ranges);
    let vecofintensitymaps: Vec<RangeAndIntensityMap> = arrayofintensityandh.0;
    let omsmap = arrayofintensityandh.1;
    let mut vecofoutputtedimages: Vec<RangeAndRGBImage> = Vec::new();
    let emptyimgtemplate: RgbImage = DynamicImage::new_rgb8(
        omsmap.arrayspec.num_rows() as u32,
        omsmap.arrayspec.num_columns() as u32,
    )
    .to_rgb8();
    for map in &vecofintensitymaps {
        vecofoutputtedimages.push(RangeAndRGBImage::new(
            emptyimgtemplate.clone(),
            map.minmz,
            map.maxmz,
            map.highestintensity,
        ));
    }
    let mut xval = 0;
    let mut now = Instant::now();
    while xval < omsmap.arrayspec.num_rows() {
        let mut yval = 0;
        if (now.elapsed().as_millis() > 500) {
            println!(
                "Generating Images: {:?}%",
                ((xval as f64 / omsmap.arrayspec.num_rows() as f64) * 10000.).ceil() / 100.
            );
            now = Instant::now();
        }
        while yval < omsmap.arrayspec.num_columns() {
            let mut iterval = 0;
            while iterval < vecofintensitymaps.len() {
                let arrayofintensitybox: &RangeAndIntensityMap = &vecofintensitymaps[iterval];
                let arrayofintensity: &Array2D<f32> = &arrayofintensitybox.arrayintensity;
                let intensitypercentage: f32 = (arrayofintensity.get(xval, yval).unwrap()
                    / arrayofintensitybox.highestintensity);
                let mut red: u8 = 0;
                let mut blue: u8 = 0;
                let mut green: u8 = 0;
                let intensitymappedas32bit: f32 = (intensitypercentage * 1020.);
                if (intensitymappedas32bit <= 255.) {
                    blue = intensitymappedas32bit.round() as u8;
                } else {
                    if (intensitymappedas32bit < 765.) {
                        blue = (255. - ((intensitymappedas32bit - 255.) / 2.)).round() as u8;
                        green = ((intensitymappedas32bit - 255.) / 2.).round() as u8;
                    } else {
                        green = 255;
                    }
                }
                let mut imgbox = &mut vecofoutputtedimages[iterval];
                let mut img = &mut imgbox.imagem;
                img.put_pixel(xval as u32, yval as u32, ::image::Rgb([red, green, blue]));
                iterval = iterval + 1;
            }
            yval = yval + 1;
        }
        xval = xval + 1;
    }
    vecofoutputtedimages
}
pub fn circularcalculatecorrilations(
    omsmapf: OptimizedMSMap,
    ranges: Vec<(f64, f64)>,
) -> Vec<(CircularCorrilationImage)> {
    let output: (Vec<RangeAndIntensityMap>, OptimizedMSMap) =
        multiintensitymapsfromOMS(omsmapf, ranges);
    let omsmap = output.1;
    let raivec = &output.0;
    let emptyimgtemplate: RgbImage = DynamicImage::new_rgb8(
        omsmap.arrayspec.num_rows() as u32,
        omsmap.arrayspec.num_columns() as u32,
    )
    .to_rgb8();
    let templatearrayofintensity: Array2D<(f32, f32)> = Array2D::filled_with(
        (0., 0.),
        omsmap.arrayspec.num_rows(),
        omsmap.arrayspec.num_columns(),
    );
    //First image is a ratio map (Blue #1, Green #2), Second image displays the consistency of the ratio
    let mut ratiovec: Vec<CircularCorrilationImage> = Vec::new();
    let mut index1 = 0;
    while (index1 < raivec.len() - 1) {
        let mut index2 = index1 + 1;
        while (index2 < raivec.len()) {
            let rai1: &RangeAndIntensityMap = &raivec[index1];
            let rai2: &RangeAndIntensityMap = &raivec[index2];
            // Avr Ratio is 1 over 2
            let mut arrayofratios: Array2D<(f32, f32)> = templatearrayofintensity.clone();
            let mut corrilavr1: f64 = 0.;
            let mut corrilavr2: f64 = 0.;
            let mut corrilavr3: f64 = 0.;
            let mut corrilavr4: f64 = 0.;
            let mut corrilavr5: f64 = 0.;
            let mut corrilavr6: f64 = 0.;
            let mut ratioavrsm = 1.;
            let mut ratioavr = 1.;
            let mut imageratio = emptyimgtemplate.clone();
            let mut image1 = emptyimgtemplate.clone();
            let mut image2 = emptyimgtemplate.clone();
            let mut image3 = emptyimgtemplate.clone();
            let mut image4 = emptyimgtemplate.clone();
            let mut image5 = emptyimgtemplate.clone();
            let mut image6 = emptyimgtemplate.clone();
            let mut xval = 0;
            let mut numreps = 0;
            let mut now = Instant::now();
            while xval < omsmap.arrayspec.num_rows() {
                let mut yval = 0;
                if (now.elapsed().as_millis() > 500) {
                    println!(
                        "Corrilating Images: {:?}%",
                        ((xval as f64 / omsmap.arrayspec.num_rows() as f64) * 10000.).ceil() / 200.
                    );
                    now = Instant::now();
                }
                while yval < omsmap.arrayspec.num_columns() {
                    let inte1 = rai1.arrayintensity.get(xval, yval).unwrap();
                    let inte2 = rai2.arrayintensity.get(xval, yval).unwrap();
                    let mut signaloverboth: f32 =
                        ((inte2 + inte1) / (rai1.highestintensity + rai2.highestintensity)) * 2.;
                    if (signaloverboth > 1.) {
                        signaloverboth = 1.;
                    }
                    let ratio1over2 = inte1 / inte2;
                    if (ratio1over2.is_nan()) {
                        arrayofratios.set(xval, yval, (0., signaloverboth));
                    } else {
                        if (ratio1over2.is_infinite()) {
                            arrayofratios.set(xval, yval, (100000., signaloverboth));
                        } else {
                            arrayofratios.set(xval, yval, (ratio1over2, signaloverboth));
                        }
                    }
                    let red: u8 = 0;
                    let mut green: u8 = 0;
                    let mut blue: u8 = 0;
                    if (inte1 > inte2) {
                        blue = (255. * signaloverboth).round() as u8;
                        green = ((inte2 / inte1) * 255.).round() as u8;
                    } else {
                        green = (255. * signaloverboth).round() as u8;
                        blue = (((inte1 / inte2) * 255.) * signaloverboth).round() as u8;
                    }
                    imageratio.put_pixel(
                        xval as u32,
                        yval as u32,
                        ::image::Rgb([red, green, blue]),
                    );
                    yval += 1;
                }
                xval += 1;
            }
            let mut totalratio: f64 = 1.;
            let mut totalsigs: f64 = 0.;
            xval = 0;
            while xval < omsmap.arrayspec.num_rows() {
                let mut yval = 0;
                if (now.elapsed().as_millis() > 500) {
                    println!(
                        "Corrilating Images: {:?}%",
                        (((xval as f64 / omsmap.arrayspec.num_rows() as f64) * 10000.).ceil()
                            / 200.)
                            + 50.
                    );
                    now = Instant::now();
                }
                while yval < omsmap.arrayspec.num_columns() {
                    let inte1 = rai1.arrayintensity.get(xval, yval).unwrap();
                    let inte2 = rai2.arrayintensity.get(xval, yval).unwrap();
                    let ratasig: &(f32, f32) = arrayofratios.get(xval, yval).unwrap();
                    let rat = ratasig.0;
                    let mut origsig: f32 = ((inte1 / rai1.highestintensity)
                        * (inte2 / rai2.highestintensity))
                        .powf(0.5);
                    let mut weightingand50cutoff = origsig * 2.;
                    if (weightingand50cutoff > 1.) {
                        weightingand50cutoff = 1.;
                    }
                    let mut weightingand25cutoff = origsig * 4.;
                    if (weightingand25cutoff > 1.) {
                        weightingand25cutoff = 1.;
                    }
                    let mut veccords: Vec<(i64, i64)> = Vec::new();
                    let mut x2val: i64 = -6;
                    let mut netoffset1: f32 = 0.;
                    let mut numinputs1: i32 = 0;
                    let mut netoffset2: f32 = 0.;
                    let mut numinputs2: i32 = 0;
                    let mut netoffset3: f32 = 0.;
                    let mut numinputs3: i32 = 0;
                    let mut netoffset4: f32 = 0.;
                    let mut numinputs4: i32 = 0;
                    let mut netoffset5: f32 = 0.;
                    let mut numinputs5: i32 = 0;
                    let mut netoffset6: f32 = 0.;
                    let mut numinputs6: i32 = 0;
                    if (rat > 0.) {
                        while (x2val <= 6) {
                            let mut y2val: i64 = -6;
                            while (y2val <= 6) {
                                if !((x2val + xval as i64) < 0 || (y2val + yval as i64) < 0) {
                                    if ((((x2val + xval as i64) as i64)
                                        < arrayofratios.num_rows() as i64)
                                        && (((y2val + yval as i64) as i64)
                                            < arrayofratios.num_columns() as i64))
                                    {
                                        let dist: f64 = ((y2val as f64).powi(2)
                                            + (x2val as f64).powi(2))
                                        .powf(0.5);
                                        if (dist <= 6. && dist > 0.) {
                                            let packval = arrayofratios.get(
                                                (x2val + xval as i64) as usize,
                                                (y2val + yval as i64) as usize,
                                            );
                                            let mut printlnval = 0.;
                                            let mut val = 0.;
                                            if (packval == None) {
                                            } else {
                                                val = packval.unwrap().0;
                                            }
                                            let mut percent = 0.;
                                            if (rat <= val) {
                                                percent = (rat / val);
                                            } else {
                                                percent = (val / rat);
                                            }
                                            // if (xval > 133) {
                                            //     println!(
                                            //         "X: {:?}, Y: {:?}, Perc: {:?}, Val: {:?}, OtherVal: {:?}",
                                            //         x2val + xval as i64,
                                            //         y2val + yval as i64,
                                            //         percent,
                                            //         val,
                                            //         rat,
                                            //     );
                                            // }
                                            if (dist <= 1.) {
                                                numinputs1 += 1;
                                                numinputs2 += 1;
                                                numinputs3 += 1;
                                                numinputs4 += 1;
                                                numinputs5 += 1;
                                                numinputs6 += 1;
                                                netoffset1 += percent;
                                                netoffset2 += percent;
                                                netoffset3 += percent;
                                                netoffset4 += percent;
                                                netoffset5 += percent;
                                                netoffset6 += percent;
                                            } else {
                                                if (dist <= 2.) {
                                                    numinputs2 += 1;
                                                    numinputs3 += 1;
                                                    numinputs4 += 1;
                                                    numinputs5 += 1;
                                                    numinputs6 += 1;
                                                    netoffset2 += percent;
                                                    netoffset3 += percent;
                                                    netoffset4 += percent;
                                                    netoffset5 += percent;
                                                    netoffset6 += percent;
                                                } else {
                                                    if (dist <= 3.) {
                                                        numinputs3 += 1;
                                                        numinputs4 += 1;
                                                        numinputs5 += 1;
                                                        numinputs6 += 1;
                                                        netoffset3 += percent;
                                                        netoffset4 += percent;
                                                        netoffset5 += percent;
                                                        netoffset6 += percent;
                                                    } else {
                                                        if (dist <= 4.) {
                                                            numinputs4 += 1;
                                                            numinputs5 += 1;
                                                            numinputs6 += 1;
                                                            netoffset4 += percent;
                                                            netoffset5 += percent;
                                                            netoffset6 += percent;
                                                        } else {
                                                            if (dist <= 5.) {
                                                                numinputs5 += 1;
                                                                numinputs6 += 1;
                                                                netoffset5 += percent;
                                                                netoffset6 += percent;
                                                            } else {
                                                                if (dist <= 6.) {
                                                                    numinputs6 += 1;
                                                                    netoffset6 += percent;
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                                y2val += 1;
                            }
                            x2val += 1;
                        }
                    }
                    let mut ratiosm: f64 = 0.;
                    if (rat > 1.) {
                        ratiosm = 1. + (1. / rat as f64);
                    } else {
                        ratiosm = rat as f64;
                    }
                    totalsigs += weightingand25cutoff as f64;
                    totalratio += ratiosm as f64 * (weightingand25cutoff) as f64;
                    let mut offset1 = 0.;
                    if (numinputs1 > 0) {
                        offset1 = (netoffset1 / numinputs1 as f32);
                    }
                    let mut offset2 = 0.;
                    if (numinputs2 > 0) {
                        offset2 = (netoffset2 / numinputs2 as f32);
                    }
                    let mut offset3 = 0.;
                    if (numinputs3 > 0) {
                        offset3 = (netoffset3 / numinputs3 as f32);
                    }
                    let mut offset4 = 0.;
                    if (numinputs4 > 0) {
                        offset4 = (netoffset4 / numinputs4 as f32);
                    }
                    let mut offset5 = 0.;
                    if (numinputs5 > 0) {
                        offset5 = (netoffset5 / numinputs5 as f32);
                    }
                    let mut offset6 = 0.;
                    if (numinputs6 > 0) {
                        offset6 = (netoffset6 / numinputs6 as f32);
                    }
                    corrilavr1 += ((offset1) * (weightingand25cutoff)) as f64;
                    corrilavr2 += ((offset2) * (weightingand25cutoff)) as f64;
                    corrilavr3 += ((offset3) * (weightingand25cutoff)) as f64;
                    corrilavr4 += ((offset4) * (weightingand25cutoff)) as f64;
                    corrilavr5 += ((offset5) * (weightingand25cutoff)) as f64;
                    corrilavr6 += ((offset6) * (weightingand25cutoff)) as f64;
                    let white1: u8 = ((255. * offset1) * weightingand50cutoff).round() as u8;
                    let white2: u8 = ((255. * offset2) * weightingand50cutoff).round() as u8;
                    let white3: u8 = ((255. * offset3) * weightingand50cutoff).round() as u8;
                    let white4: u8 = ((255. * offset4) * weightingand50cutoff).round() as u8;
                    let white5: u8 = ((255. * offset5) * weightingand50cutoff).round() as u8;
                    let white6: u8 = ((255. * offset6) * weightingand50cutoff).round() as u8;
                    image1.put_pixel(
                        xval as u32,
                        yval as u32,
                        ::image::Rgb([white1, white1, white1]),
                    );
                    image2.put_pixel(
                        xval as u32,
                        yval as u32,
                        ::image::Rgb([white2, white2, white2]),
                    );
                    image3.put_pixel(
                        xval as u32,
                        yval as u32,
                        ::image::Rgb([white3, white3, white3]),
                    );
                    image4.put_pixel(
                        xval as u32,
                        yval as u32,
                        ::image::Rgb([white4, white4, white4]),
                    );
                    image5.put_pixel(
                        xval as u32,
                        yval as u32,
                        ::image::Rgb([white5, white5, white5]),
                    );
                    image6.put_pixel(
                        xval as u32,
                        yval as u32,
                        ::image::Rgb([white6, white6, white6]),
                    );
                    yval += 1;
                    numreps += 1;
                }
                xval += 1;
            }
            ratioavrsm = (totalratio / totalsigs);
            if (ratioavrsm > 1.) {
                ratioavr = (1. / (ratioavrsm - 1.))
            } else {
                ratioavr = ratioavrsm;
            }
            corrilavr1 = (corrilavr1 / totalsigs);
            corrilavr2 = (corrilavr2 / totalsigs);
            corrilavr3 = (corrilavr3 / totalsigs);
            corrilavr4 = (corrilavr4 / totalsigs);
            corrilavr5 = (corrilavr5 / totalsigs);
            corrilavr6 = (corrilavr6 / totalsigs);

            ratiovec.push(CircularCorrilationImage::new(
                RatioRangeAndRGBImage::new(
                    imageratio,
                    rai1.minmz,
                    rai1.maxmz,
                    rai2.minmz,
                    rai2.maxmz,
                    rai1.highestintensity,
                    rai2.highestintensity,
                ),
                RatioRangeAndRGBImage::new(
                    image1,
                    rai1.minmz,
                    rai1.maxmz,
                    rai2.minmz,
                    rai2.maxmz,
                    rai1.highestintensity,
                    rai2.highestintensity,
                ),
                RatioRangeAndRGBImage::new(
                    image2,
                    rai1.minmz,
                    rai1.maxmz,
                    rai2.minmz,
                    rai2.maxmz,
                    rai1.highestintensity,
                    rai2.highestintensity,
                ),
                RatioRangeAndRGBImage::new(
                    image3,
                    rai1.minmz,
                    rai1.maxmz,
                    rai2.minmz,
                    rai2.maxmz,
                    rai1.highestintensity,
                    rai2.highestintensity,
                ),
                RatioRangeAndRGBImage::new(
                    image4,
                    rai1.minmz,
                    rai1.maxmz,
                    rai2.minmz,
                    rai2.maxmz,
                    rai1.highestintensity,
                    rai2.highestintensity,
                ),
                RatioRangeAndRGBImage::new(
                    image5,
                    rai1.minmz,
                    rai1.maxmz,
                    rai2.minmz,
                    rai2.maxmz,
                    rai1.highestintensity,
                    rai2.highestintensity,
                ),
                RatioRangeAndRGBImage::new(
                    image6,
                    rai1.minmz,
                    rai1.maxmz,
                    rai2.minmz,
                    rai2.maxmz,
                    rai1.highestintensity,
                    rai2.highestintensity,
                ),
                corrilavr1,
                corrilavr2,
                corrilavr3,
                corrilavr4,
                corrilavr5,
                corrilavr6,
                ratioavr,
                "Blue for range 1, green for range 2".to_owned(),
            ));
            index2 += 1;
        }
        index1 += 1;
    }
    ratiovec
}
pub fn unoptimcalculatecorrilations(
    omsmapf: OptimizedMSMap,
    ranges: Vec<(f64, f64)>,
) -> Vec<(CorrilationImage)> {
    let output: (Vec<RangeAndIntensityMap>, OptimizedMSMap) =
        multiintensitymapsfromOMS(omsmapf, ranges);
    let omsmap = output.1;
    let raivec = &output.0;
    let emptyimgtemplate: RgbImage = DynamicImage::new_rgb8(
        omsmap.arrayspec.num_rows() as u32,
        omsmap.arrayspec.num_columns() as u32,
    )
    .to_rgb8();
    let templatearrayofintensity: Array2D<(f32, f32)> = Array2D::filled_with(
        (0., 0.),
        omsmap.arrayspec.num_rows(),
        omsmap.arrayspec.num_columns(),
    );
    //First image is a ratio map (Blue #1, Green #2), Second image displays the consistency of the ratio
    let mut ratiovec: Vec<CorrilationImage> = Vec::new();
    let mut index1 = 0;
    while (index1 < raivec.len() - 1) {
        let mut index2 = index1 + 1;
        while (index2 < raivec.len()) {
            let rai1: &RangeAndIntensityMap = &raivec[index1];
            let rai2: &RangeAndIntensityMap = &raivec[index2];
            // Avr Ratio is 1 over 2
            let mut arrayofratios: Array2D<(f32, f32)> = templatearrayofintensity.clone();
            let mut corrilavr: f64 = 0.;
            let mut ratioavrsm = 1.;
            let mut ratioavr = 1.;
            let mut imageratio = emptyimgtemplate.clone();
            let mut imagecorrilval = emptyimgtemplate.clone();
            let mut xval = 0;
            let mut numreps = 0;
            let mut now = Instant::now();
            while xval < omsmap.arrayspec.num_rows() {
                let mut yval = 0;
                if (now.elapsed().as_millis() > 500) {
                    println!(
                        "Corrilating Images: {:?}%",
                        ((xval as f64 / omsmap.arrayspec.num_rows() as f64) * 10000.).ceil() / 200.
                    );
                    now = Instant::now();
                }
                while yval < omsmap.arrayspec.num_columns() {
                    let inte1 = rai1.arrayintensity.get(xval, yval).unwrap();
                    let inte2 = rai2.arrayintensity.get(xval, yval).unwrap();
                    let mut signaloverboth: f32 =
                        ((inte2 + inte1) / (rai1.highestintensity + rai2.highestintensity)) * 2.;
                    if (signaloverboth > 1.) {
                        signaloverboth = 1.;
                    }
                    let ratio1over2 = inte1 / inte2;
                    if (ratio1over2.is_nan()) {
                        arrayofratios.set(xval, yval, (0., signaloverboth));
                    } else {
                        if (ratio1over2.is_infinite()) {
                            arrayofratios.set(xval, yval, (100000., signaloverboth));
                        } else {
                            arrayofratios.set(xval, yval, (ratio1over2, signaloverboth));
                        }
                    }
                    let red: u8 = 0;
                    let mut green: u8 = 0;
                    let mut blue: u8 = 0;
                    if (inte1 > inte2) {
                        blue = (255. * signaloverboth).round() as u8;
                        green = ((inte2 / inte1) * 255.).round() as u8;
                    } else {
                        green = (255. * signaloverboth).round() as u8;
                        blue = (((inte1 / inte2) * 255.) * signaloverboth).round() as u8;
                    }
                    imageratio.put_pixel(
                        xval as u32,
                        yval as u32,
                        ::image::Rgb([red, green, blue]),
                    );
                    yval += 1;
                }
                xval += 1;
            }
            let mut totalcorril: f64 = 1.;
            let mut totalratio: f64 = 1.;
            let mut totalsigs: f64 = 0.;
            xval = 0;
            while xval < omsmap.arrayspec.num_rows() {
                let mut yval = 0;
                if (now.elapsed().as_millis() > 500) {
                    println!(
                        "Corrilating Images: {:?}%",
                        (((xval as f64 / omsmap.arrayspec.num_rows() as f64) * 10000.).ceil()
                            / 200.)
                            + 50.
                    );
                    now = Instant::now();
                }
                while yval < omsmap.arrayspec.num_columns() {
                    let inte1 = rai1.arrayintensity.get(xval, yval).unwrap();
                    let inte2 = rai2.arrayintensity.get(xval, yval).unwrap();
                    let ratasig: &(f32, f32) = arrayofratios.get(xval, yval).unwrap();
                    let rat = ratasig.0;
                    let mut origsig: f32 = ((inte1 / rai1.highestintensity)
                        * (inte2 / rai2.highestintensity))
                        .powf(0.5);
                    let mut weightingand50cutoff = origsig * 2.;
                    if (weightingand50cutoff > 1.) {
                        weightingand50cutoff = 1.;
                    }
                    let mut weightingand25cutoff = origsig * 4.;
                    if (weightingand25cutoff > 1.) {
                        weightingand25cutoff = 1.;
                    }
                    let mut veccords: Vec<(i64, i64)> = Vec::new();
                    veccords.push((xval as i64 + 1, yval as i64));
                    veccords.push((xval as i64 - 1, yval as i64));
                    veccords.push((xval as i64, yval as i64 + 1));
                    veccords.push((xval as i64, yval as i64 - 1));
                    veccords.push((xval as i64 + 1, yval as i64 + 1));
                    veccords.push((xval as i64 - 1, yval as i64 + 1));
                    veccords.push((xval as i64 + 1, yval as i64 - 1));
                    veccords.push((xval as i64 - 1, yval as i64 - 1));
                    let mut netoffset: f32 = 0.;
                    let mut numinputs: i32 = 0;
                    if (rat > 0.) {
                        for cord in veccords {
                            if !(cord.0 < 0 || cord.1 < 0) {
                                if (cord.0 < arrayofratios.num_rows() as i64
                                    && cord.1 < arrayofratios.num_columns() as i64)
                                {
                                    numinputs += 1;
                                    let val = arrayofratios
                                        .get(cord.0 as usize, cord.1 as usize)
                                        .unwrap()
                                        .0;
                                    if (val > 0.) {
                                        if (rat <= val) {
                                            netoffset += (rat / val);
                                        } else {
                                            netoffset += (val / rat);
                                        }
                                    }
                                }
                            }
                        }
                    }
                    let mut ratiosm: f64 = 0.;
                    if (rat > 1.) {
                        ratiosm = 1. + (1. / rat as f64);
                    } else {
                        ratiosm = rat as f64;
                    }
                    totalsigs += weightingand25cutoff as f64;
                    totalratio += ratiosm as f64 * (weightingand25cutoff) as f64;
                    let mut offset = 0.;
                    if (numinputs > 0) {
                        offset = (netoffset / numinputs as f32);
                    }
                    corrilavr += ((offset) * (weightingand25cutoff)) as f64;
                    let white: u8 = ((255. * offset) * weightingand50cutoff).round() as u8;
                    let red: u8 = white;
                    let mut green: u8 = white;
                    let mut blue: u8 = white;
                    imagecorrilval.put_pixel(
                        xval as u32,
                        yval as u32,
                        ::image::Rgb([red, green, blue]),
                    );
                    yval += 1;
                    numreps += 1;
                }
                xval += 1;
            }
            ratioavrsm = (totalratio / totalsigs);
            if (ratioavrsm > 1.) {
                ratioavr = (1. / (ratioavrsm - 1.))
            } else {
                ratioavr = ratioavrsm;
            }
            corrilavr = (totalcorril / totalsigs);
            ratiovec.push(CorrilationImage::new(
                RatioRangeAndRGBImage::new(
                    imageratio,
                    rai1.minmz,
                    rai1.maxmz,
                    rai2.minmz,
                    rai2.maxmz,
                    rai1.highestintensity,
                    rai2.highestintensity,
                ),
                RatioRangeAndRGBImage::new(
                    imagecorrilval,
                    rai1.minmz,
                    rai1.maxmz,
                    rai2.minmz,
                    rai2.maxmz,
                    rai1.highestintensity,
                    rai2.highestintensity,
                ),
                corrilavr,
                ratioavr,
                "Blue for range 1, green for range 2".to_owned(),
            ));
            index2 += 1;
        }
        index1 += 1;
    }
    ratiovec
}
pub fn multiintensitymapsfromOMS(
    omsmap: OptimizedMSMap,
    ranges: Vec<(f64, f64)>,
) -> (Vec<RangeAndIntensityMap>, OptimizedMSMap) {
    let mut setofintensitymaps: Vec<RangeAndIntensityMap> = Vec::new();
    let templatearrayofintensity: Array2D<f32> = Array2D::filled_with(
        0.,
        omsmap.arrayspec.num_rows(),
        omsmap.arrayspec.num_columns(),
    );
    for minmaxmz in ranges {
        setofintensitymaps.push(RangeAndIntensityMap::new(
            templatearrayofintensity.clone(),
            minmaxmz.0,
            minmaxmz.1,
            1.,
        ))
    }
    let mut xval = 1;
    let mut now = Instant::now();
    while xval < omsmap.arrayspec.num_rows() {
        let mut yval = 1;
        if (now.elapsed().as_millis() > 500) {
            println!(
                "Generating INMaps For Image: {:?}%",
                ((xval as f64 / omsmap.arrayspec.num_rows() as f64) * 10000.).ceil() / 100.
            );
            now = Instant::now();
        }
        while yval < omsmap.arrayspec.num_columns() {
            let spec: &OSpectrum = omsmap.arrayspec.get(xval, yval).unwrap();
            for peek in &spec.peeks {
                let mut iterval = 0;
                while iterval < setofintensitymaps.len() {
                    let mut rai = &mut setofintensitymaps[iterval];
                    let mut arrayofintensity = &mut (rai).arrayintensity;
                    let currentsetval = arrayofintensity.get(xval, yval).unwrap()
                        + peekeval(spec, peek, rai.minmz, rai.maxmz) as f32;
                    arrayofintensity.set(xval, yval, currentsetval).unwrap();
                    iterval = iterval + 1;
                    if (currentsetval > rai.highestintensity) {
                        rai.highestintensity = currentsetval;
                    }
                }
            }
            yval = yval + 1;
        }
        xval = xval + 1;
    }
    (setofintensitymaps, omsmap)
}
pub fn intensitymapfromOMS(
    omsmap: &OptimizedMSMap,
    minrange: f64,
    maxrange: f64,
) -> (Array2D<f64>, f64) {
    let mut xval = 1;
    let mut arrayofintensity: Array2D<f64> = Array2D::filled_with(
        0.,
        omsmap.arrayspec.num_rows(),
        omsmap.arrayspec.num_columns(),
    );
    let mut highestsignal: f64 = 1.;
    while xval < omsmap.arrayspec.num_rows() {
        let mut yval = 1;
        while yval < omsmap.arrayspec.num_columns() {
            let spec: &OSpectrum = omsmap.arrayspec.get(xval, yval).unwrap();
            for peek in &spec.peeks {
                arrayofintensity.set(
                    xval,
                    yval,
                    arrayofintensity.get(xval, yval).unwrap()
                        + peekeval(spec, peek, minrange, maxrange),
                );
            }
            if (arrayofintensity.get(xval, yval).unwrap().clone() > highestsignal) {
                highestsignal = arrayofintensity.get(xval, yval).unwrap().clone();
            };
            yval = yval + 1;
        }
        xval = xval + 1;
    }
    (arrayofintensity, highestsignal)
}

pub fn unoptimizedparseimzml(
    ibdpath: PathBuf,
    imzmlpath: PathBuf,
    gl: &mut GlGraphics,
    window: &mut Window,
    events: &mut Events,
) -> UnoptimizedMSMap {
    let mut vecobytes: Vec<u8> = get_file_as_byte_vec(ibdpath);
    // I copy-pasted this code from StackOverflow without reading the answer
    // surrounding it that told me to write a comment explaining why this code
    // is actually safe for my own use case.
    let root: minidom::Element = std::fs::read_to_string(imzmlpath)
        .expect("ERROR can not read IMZML file")
        .parse()
        .unwrap();

    let mut specvec: Vec<Spectrum> = Vec::new();
    let mut averagetime: Vec<u128> = Vec::new();
    let mut codesegmentaveragetime: Vec<u128> = Vec::new();
    let mut xsize: usize = 0;
    let mut ysize: usize = 0;
    let sizesettings: &minidom::Element = root
        .get_child("scanSettingsList", NSChoice::Any)
        .unwrap()
        .get_child("scanSettings", NSChoice::Any)
        .unwrap();
    for settingmod in sizesettings.children() {
        if (settingmod.attr("name").unwrap() == "max count of pixel x") {
            xsize = settingmod.attr("value").unwrap().parse::<usize>().unwrap();
        }
        if (settingmod.attr("name").unwrap() == "max count of pixel y") {
            ysize = settingmod.attr("value").unwrap().parse::<usize>().unwrap();
        }
    }
    let rootspeclen: usize = root
        .get_child("run", NSChoice::Any)
        .unwrap()
        .get_child("spectrumList", NSChoice::Any)
        .unwrap()
        .children()
        .count();
    for (i, child) in root
        .get_child("run", NSChoice::Any)
        .unwrap()
        .get_child("spectrumList", NSChoice::Any)
        .unwrap()
        .children()
        .enumerate()
    {
        if (i as f64 / 500.).ceil() == i as f64 / 500. {
            loadingscreen(
                &format!(
                    "Reading {:?}% Complete",
                    ((i as f64 * 10000.) / (rootspeclen as f64)).ceil() / 100.
                ),
                gl,
                window,
                events,
            )
        }
        let spectimer = Instant::now();

        let mut xval: i64 = 0;
        let mut yval: i64 = 0;
        let mut m_over_zscanning = true;
        let mut intensityvec: Vec<f32> = Vec::new();
        let mut m_over_zvec: Vec<f64> = Vec::new();
        let mut bvec: Vec<Mzplusintensity> = Vec::new();
        let scan = child
            .get_child("scanList", NSChoice::Any)
            .unwrap()
            .get_child("scan", NSChoice::Any)
            .unwrap();
        let bidaarli = child
            .get_child("binaryDataArrayList", NSChoice::Any)
            .unwrap();
        for scanchild in scan.children() {
            if !(scanchild.attr("name") == None) {
                if (scanchild.attr("name").unwrap() == "position x") {
                    xval = scanchild.attr("value").unwrap().parse::<i64>().unwrap();
                }
                if (scanchild.attr("name").unwrap() == "position y") {
                    yval = scanchild.attr("value").unwrap().parse::<i64>().unwrap();
                }
            }
        }
        for bidaar in bidaarli.children() {
            let mut objlen: i64 = 0;
            let mut bitoffset: i64 = 0;
            let mut bitlen: i64 = 0;
            for bidaarchild in bidaar.children() {
                if !(bidaarchild.attr("ref") == None) {
                    if (bidaarchild.attr("ref").unwrap() == "mzArray") {
                        m_over_zscanning = true;
                    } else {
                        m_over_zscanning = false;
                    }
                } else {
                    if !(bidaarchild.attr("name") == None) {
                        if (bidaarchild.attr("name").unwrap() == "external array length") {
                            objlen = bidaarchild.attr("value").unwrap().parse::<i64>().unwrap();
                        }
                        if (bidaarchild.attr("name").unwrap() == "external offset") {
                            bitoffset = bidaarchild.attr("value").unwrap().parse::<i64>().unwrap();
                        }
                        if (bidaarchild.attr("name").unwrap() == "external encoded length") {
                            bitlen = bidaarchild.attr("value").unwrap().parse::<i64>().unwrap();
                        }
                    }
                }
            }
            let mut bitpos: i64 = bitoffset;
            let segmentspectimer = Instant::now();
            if (m_over_zscanning) {
                while (bitpos < (bitoffset + bitlen)) {
                    let mut rdrr = vec![
                        vecobytes[bitpos as usize + 0],
                        vecobytes[bitpos as usize + 1],
                        vecobytes[bitpos as usize + 2],
                        vecobytes[bitpos as usize + 3],
                        vecobytes[bitpos as usize + 4],
                        vecobytes[bitpos as usize + 5],
                        vecobytes[bitpos as usize + 6],
                        vecobytes[bitpos as usize + 7],
                    ];
                    let mut rdr = Cursor::new(&rdrr);
                    let ff: f64 = rdr.read_f64::<LittleEndian>().unwrap();

                    m_over_zvec.push(ff);
                    bitpos = bitpos + 8;
                }
            } else {
                while (bitpos < (bitoffset + bitlen)) {
                    let mut rdrr = vec![
                        vecobytes[bitpos as usize + 0],
                        vecobytes[bitpos as usize + 1],
                        vecobytes[bitpos as usize + 2],
                        vecobytes[bitpos as usize + 3],
                    ];
                    let mut rdr = Cursor::new(&rdrr);
                    let ff: f32 = rdr.read_f32::<LittleEndian>().unwrap();

                    intensityvec.push(ff);
                    bitpos = bitpos + 4;
                }
            }
            codesegmentaveragetime.push(segmentspectimer.elapsed().as_nanos());
        }
        let mut i: i64 = 0;
        while i < m_over_zvec.len() as i64 {
            if ((xval == 73) && (yval == 114))
                && (m_over_zvec[i as usize] > 400.)
                && (m_over_zvec[i as usize] < 500.)
            {
                println!(
                    "Mz: {:?}, Intensity: {:?}, Xval: {:?}, Yval: {:?}",
                    m_over_zvec[i as usize], intensityvec[i as usize], xval, yval
                );
            }
            bvec.push(Mzplusintensity::new(
                m_over_zvec[i as usize],
                intensityvec[i as usize],
            ));
            i = i + 1;
        }
        specvec.push(Spectrum::new(bvec, xval, yval));
        averagetime.push(spectimer.elapsed().as_nanos());
    }
    let mut total: u128 = 0;
    for timeu128 in averagetime {
        total = total + timeu128;
    }
    println!(
        "Average TIME Per Spectrum: {:?} Microseconds",
        ((total as f64 / rootspeclen as f64) / 10.).ceil() / 100.
    );
    total = 0;
    for timeu128 in codesegmentaveragetime {
        total = total + timeu128;
    }
    println!(
        "Average TIME Per BIANARY READINGS in Spectrum: {:?} Microseconds",
        ((total as f64 / rootspeclen as f64) / 10.).ceil() / 100.
    );
    return UnoptimizedMSMap::new(specvec, xsize as i64, ysize as i64);
}
