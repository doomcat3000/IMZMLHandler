use crate::imzml_types::*;
use crate::parsers::*;
use array2d::*;
use byteorder::{BigEndian, LittleEndian, ReadBytesExt, WriteBytesExt};
use image::io::Reader;
use image::DynamicImage;
use image::{GenericImage, GenericImageView, ImageBuffer, Pixel, Pixels, RgbImage};
use minidom::NSChoice;
use std::collections::HashMap;
use std::fs::File;
use std::io::Cursor;
use std::io::Read;
use std::path::Path;
use std::path::PathBuf;
use std::str::*;
use std::time::Instant;
use piston::event_loop::{EventSettings, Events};
use piston::input::{ButtonEvent, Event, RenderEvent};
use piston::window::WindowSettings;
use glutin_window::GlutinWindow as Window;
use std::*;
use opengl_graphics::*;
use opengl_graphics::{GlGraphics, OpenGL};
pub fn genenzymepresenceimgs(
    omsmap: &OptimizedMSMap,
    setsofranges: Vec<((f64, f64), (f64, f64))>,
    mut gl: &mut GlGraphics,
    window: &mut Window,
    events: &mut Events,
) -> Vec<(RgbImage)> {
    let mut xval = 0;
    let mut hashofintensitymaps: HashMap<i64, V2RangeAndIntensityMap> = HashMap::new();
    // List of enzymes that convert to and from a given molecule
    // List of enzymes that can turn a molecule into something new
    let mut convertfrommolec: HashMap<i64, Vec<((f64, f64))>> = HashMap::new();
    let mut avermolecconsistancy: HashMap<i64, f64> = HashMap::new();
    let mut advancedenzymecorrilval: HashMap<i64, f64> = HashMap::new();
    // List of enzymes that can turn something into a molecule
    let mut converttomolec: HashMap<i64, Vec<((f64, f64))>> = HashMap::new();
    let templatearrayofintensity: Array2D<f32> = Array2D::filled_with(
        0.,
        omsmap.arrayspec.num_rows(),
        omsmap.arrayspec.num_columns(),
    );
    let mut keylist: Vec<i64> = Vec::new();
    for enzyme in &setsofranges {
        let startintmap = V2RangeAndIntensityMap::new(
            templatearrayofintensity.clone(),
            enzyme.0.0,
            enzyme.0.1,
            1.,
            0.,
        );
        let endintmap = V2RangeAndIntensityMap::new(
            templatearrayofintensity.clone(),
            enzyme.1.0,
            enzyme.1.1,
            1.,
            0.,
        );
        let startkey = (enzyme.0.0 * 1000000.) as i64;
        let endkey = (enzyme.1.0 * 1000000.) as i64;
        let enzymekey = (enzyme.1.0 * 1010000.) as i64 + (enzyme.1.1 * 1020000.) as i64 + (enzyme.0.0 * 1030000.) as i64 + (enzyme.0.1 * 1040000.) as i64;
        avermolecconsistancy.insert(enzymekey, 0.);
        advancedenzymecorrilval.insert(enzymekey, 0.);
        if !hashofintensitymaps.contains_key(&startkey) {
            keylist.push(startkey);
            hashofintensitymaps.insert(startkey, startintmap);
        }
        if !hashofintensitymaps.contains_key(&endkey) {
            keylist.push(endkey);
            hashofintensitymaps.insert(endkey, endintmap);
        }
    }
    let mut totalpoints = 0;
    let mut genhighval = 1.;
    while xval < omsmap.arrayspec.num_rows() {
        let mut yval = 0;
        while yval < omsmap.arrayspec.num_columns() {
            let spec: &OSpectrum = omsmap.arrayspec.get(xval, yval).unwrap();
            for peek in &spec.peeks {
                let mut iterval = 0;
                while iterval < hashofintensitymaps.len() {
                    let mut rai = &mut hashofintensitymaps.get_mut(&keylist[iterval]).unwrap();
                    let mut arrayofintensity = &mut (rai).arrayintensity;
                    let currentsetval = arrayofintensity.get(xval, yval).unwrap()
                        + peekeval(spec, peek, rai.minmz, rai.maxmz) as f32;
                    arrayofintensity.set(xval, yval, currentsetval).unwrap();
                    iterval = iterval + 1;
                    if (currentsetval > rai.highestintensity) {
                        rai.highestintensity = currentsetval;
                    }
                    rai.avervalnotrelativetointensity += currentsetval.powi(2);
                    if (currentsetval > genhighval) {
                        genhighval = currentsetval;
                    }
                }
            }
            totalpoints += 1;
            yval += 4;
        }
        xval += 4;
    }
    let mut iterval = 0;
    let mut highestavrval = 0.;
    while iterval < hashofintensitymaps.len() {
        let mut rai = &mut hashofintensitymaps.get_mut(&keylist[iterval]).unwrap();
        rai.avervalnotrelativetointensity =
            (rai.avervalnotrelativetointensity / (totalpoints as f32)).powf(0.5);
        println!(
            "Avrval: {:?}, for mz {:?} to {:?}",
            rai.avervalnotrelativetointensity, rai.minmz, rai.maxmz
        );
        iterval += 1;
        if (rai.avervalnotrelativetointensity > highestavrval) {
            highestavrval = rai.avervalnotrelativetointensity;
        }
    }
    let mut crossenzymeweighting = 0.;
    for enzyme in &setsofranges {
        let mut x2val = 0;
        let startkey = (enzyme.0.0 * 1000000.) as i64;
        let endkey = (enzyme.1.0 * 1000000.) as i64;
        let enzymekey = (enzyme.1.0 * 1010000.) as i64 + (enzyme.1.1 * 1020000.) as i64 + (enzyme.0.0 * 1030000.) as i64 + (enzyme.0.1 * 1040000.) as i64;
        let mut startmap: &V2RangeAndIntensityMap = hashofintensitymaps.get(&startkey).unwrap();
        let mut endmap: &V2RangeAndIntensityMap = hashofintensitymaps.get(&endkey).unwrap();
        let mut totalavr = 0.;
        let mut totalweighting = 0.;
        while x2val < omsmap.arrayspec.num_rows() {
            let mut y2val = 0;
            while y2val < omsmap.arrayspec.num_columns() {
                let intensitystart = startmap.arrayintensity.get(x2val, y2val).unwrap();
                let intensityend = endmap.arrayintensity.get(x2val, y2val).unwrap();
                let mut ratio = 0.;
                if !(intensitystart == &0.) {
                    if (intensitystart > intensityend) {
                        ratio = intensityend / intensitystart;
                    } else {
                        ratio = 1.7 - ((intensitystart / intensityend) * 0.7);
                    }
                }
                let weighting = intensitystart;
                totalavr = totalavr + (ratio * weighting);
                totalweighting += weighting;
                y2val += 4;
            }
            x2val += 4;
        }
        avermolecconsistancy.insert(enzymekey, totalavr as f64 / totalweighting as f64);
    }
    let mut setofvalidenzymeranges: Vec<((f64, f64), (f64, f64))> = Vec::new();
    for enzyme in &setsofranges {
        let enzymekey = (enzyme.1.0 * 1010000.) as i64 + (enzyme.1.1 * 1020000.) as i64 + (enzyme.0.0 * 1030000.) as i64 + (enzyme.0.1 * 1040000.) as i64;
        let consistency = avermolecconsistancy[&enzymekey];
        println!(
            "AvrRatio: {:?}, for mz ( {:?} )",
            consistency, enzyme
        );
        if(consistency > 0.01){
            setofvalidenzymeranges.push(enzyme.clone());
        }
    }
    let mut hashofvalidenz: HashMap<i64, V2RangeAndIntensityMap> = HashMap::new();
    let mut mapforcorrilationmaps: HashMap<i64, i64> = HashMap::new();
    let mut corrilationmaps: Vec<Array2D<f32>> = Vec::new();
    let mut hashofvalidelements: HashMap<i64, (f64, f64)> = HashMap::new();
    for enzyme in &setofvalidenzymeranges {
        let enzymekey = (enzyme.1.0 * 1010000.) as i64 + (enzyme.1.1 * 1020000.) as i64 + (enzyme.0.0 * 1030000.) as i64 + (enzyme.0.1 * 1040000.) as i64;
        let startintmap = V2RangeAndIntensityMap::new(
            templatearrayofintensity.clone(),
            enzyme.0.0,
            enzyme.0.1,
            1.,
            0.,
        );
        let endintmap = V2RangeAndIntensityMap::new(
            templatearrayofintensity.clone(),
            enzyme.1.0,
            enzyme.1.1,
            1.,
            0.,
        );
        mapforcorrilationmaps.insert(enzymekey, corrilationmaps.len() as i64);
        corrilationmaps.push(templatearrayofintensity.clone());
        let startkey = (enzyme.0.0 * 1000000.) as i64;
        let endkey = (enzyme.1.0 * 1000000.) as i64;
        if !hashofvalidenz.contains_key(&startkey) {
            hashofvalidenz.insert(startkey, startintmap);
        }
        if !hashofvalidenz.contains_key(&endkey) {
            hashofvalidenz.insert(endkey, endintmap);
        }
        if !hashofvalidelements.contains_key(&startkey) {
            hashofvalidelements.insert(startkey, enzyme.0);
        }
        if !hashofvalidelements.contains_key(&endkey) {
            hashofvalidelements.insert(endkey, enzyme.1);
        }
    }
    xval = 0;
    while xval < omsmap.arrayspec.num_rows() {
        let mut yval = 0;
        while yval < omsmap.arrayspec.num_columns() {
            let spec: &OSpectrum = omsmap.arrayspec.get(xval, yval).unwrap();
            for peek in &spec.peeks {
                let mut iterval = 0;
                while iterval < hashofvalidenz.len() {
                    let mut rai = &mut hashofvalidenz.get_mut(&keylist[iterval]).unwrap();
                    let mut arrayofintensity = &mut (rai).arrayintensity;
                    let currentsetval = arrayofintensity.get(xval, yval).unwrap()
                        + peekeval(spec, peek, rai.minmz, rai.maxmz) as f32;
                    arrayofintensity.set(xval, yval, currentsetval).unwrap();
                    iterval = iterval + 1;
                    if (currentsetval > rai.highestintensity) {
                        rai.highestintensity = currentsetval;
                    }
                    rai.avervalnotrelativetointensity += currentsetval.powi(2);
                    if (currentsetval > genhighval) {
                        genhighval = currentsetval;
                    }
                }
            }
            totalpoints += 1;
            yval += 1;
        }
        xval += 1;
    }
    xval = 0;
    while xval < omsmap.arrayspec.num_rows() {
        let mut yval = 0;
        while yval < omsmap.arrayspec.num_columns() {
            for enzyme in &setofvalidenzymeranges {
                let startkey = (enzyme.0.0 * 1000000.) as i64;
                let endkey = (enzyme.1.0 * 1000000.) as i64;
                let mut startrai: &V2RangeAndIntensityMap = hashofvalidenz.get(&startkey).unwrap();
                let mut endrai = hashofvalidenz.get(&endkey).unwrap();
                let inte1 = startrai.arrayintensity.get(xval, yval).unwrap();
                let inte2 = endrai.arrayintensity.get(xval, yval).unwrap();
                let mut rat = 0.;
                if (inte1 > inte2) {
                    rat = inte2 / inte1;
                }else{
                    rat = inte1 / inte2;
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
                            if (cord.0 < startrai.arrayintensity.num_rows() as i64
                                && cord.1 < startrai.arrayintensity.num_columns() as i64)
                            {
                                numinputs += 1;
                                let inteval1 = startrai.arrayintensity.get(cord.0 as usize, cord.1 as usize).unwrap();
                                let inteval2 = endrai.arrayintensity.get(cord.0 as usize, cord.1 as usize).unwrap();
                                let mut val = 0.;
                                if (inteval1 > inteval2) {
                                    val = inteval2 / inteval1;
                                }else{
                                    val = inteval1 / inteval2;
                                }
                                //println!("Rat: {:?}, Val: {:?}", rat, val);
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
                let mut averconsistancy = 0.;
                if (numinputs > 0){
                    averconsistancy = netoffset / numinputs as f32;
                }
                //println!("Netoffset: {:?}, Numinputs: {:?}, AvrCons: {:?}", netoffset, numinputs, averconsistancy);
                let enzymekey = (enzyme.1.0 * 1010000.) as i64 + (enzyme.1.1 * 1020000.) as i64 + (enzyme.0.0 * 1030000.) as i64 + (enzyme.0.1 * 1040000.) as i64;
                let corrilmapentry = &mut corrilationmaps[mapforcorrilationmaps[&enzymekey] as usize];
                corrilmapentry.set(xval, yval, averconsistancy).unwrap();
                advancedenzymecorrilval.insert(enzymekey, advancedenzymecorrilval[&enzymekey] + averconsistancy as f64);                
            }
            totalpoints += 1;
            yval += 1;
        }
        xval += 1;
    }
    for enzyme in &setofvalidenzymeranges {
        let enzymekey = (enzyme.1.0 * 1010000.) as i64 + (enzyme.1.1 * 1020000.) as i64 + (enzyme.0.0 * 1030000.) as i64 + (enzyme.0.1 * 1040000.) as i64;
        let mut consistancyofenzyme = advancedenzymecorrilval[&enzymekey] / totalpoints as f64;
        println!("AvrConsistency: {:?},  Total Consistancy: {:?}, MZranges: {:?}", consistancyofenzyme, advancedenzymecorrilval[&enzymekey], enzyme);
        advancedenzymecorrilval.insert(enzymekey, consistancyofenzyme);       
    }
    let mut cloneoelemlist = Vec::new();
    for (k, v) in  hashofvalidelements.iter() {
        cloneoelemlist.push(v.clone());
    }
    for enzyme in &setofvalidenzymeranges {
        let startkey = (enzyme.0.0 * 1000000.) as i64;
        let endkey = (enzyme.1.0 * 1000000.) as i64;
        if !(convertfrommolec.contains_key(&endkey)) {
            convertfrommolec.insert(endkey, Vec::new());
        }
        if (convertfrommolec.contains_key(&startkey)) {
            convertfrommolec
                .get_mut(&startkey)
                .unwrap()
                .push(enzyme.1.clone());
        } else {
            convertfrommolec.insert(startkey, Vec::new());
            convertfrommolec
                .get_mut(&startkey)
                .unwrap()
                .push(enzyme.1.clone());
        }
        if !(converttomolec.contains_key(&startkey)) {
            converttomolec.insert(startkey, Vec::new());
        }
        if (converttomolec.contains_key(&endkey)) {
            converttomolec
                .get_mut(&endkey)
                .unwrap()
                .push(enzyme.0.clone());
        } else {
            converttomolec.insert(endkey, Vec::new());
            converttomolec
                .get_mut(&endkey)
                .unwrap()
                .push(enzyme.0.clone());
        }
    }
    let mut index = 0;
    let mut indexofs = 0;
    let mut listofroots: Vec<(f64, f64)> =  Vec::new();
    let mut listofunroots: Vec<(f64, f64)> =  Vec::new();
    let elem = cloneoelemlist[index];
     let elemkey = (elem.0 * 1000000.) as i64;
    listofroots = treeify(&convertfrommolec, &converttomolec, Vec::new(), cloneoelemlist.clone(), cloneoelemlist[0], true).0;
    listofunroots = untreeify(&convertfrommolec, &converttomolec, Vec::new(), cloneoelemlist.clone(), cloneoelemlist[0], true).0;
    for elemroot in listofroots {
        let elemrootkey = (elemroot.0 * 1000000.) as i64;
        let toelem = converttomolec.get(&elemrootkey);
    }
    Vec::new()
}
pub fn untreeify(
    convertfrommolec: &HashMap<i64, Vec<(f64, f64)>>,
    converttomolec: &HashMap<i64, Vec<(f64, f64)>>,
    vecofrootst: Vec<(f64, f64)>,
    elemlistt: Vec<(f64, f64)>,
    elem: (f64, f64),
    maintreemode: bool,
) -> (Vec<(f64, f64)>, Vec<(f64, f64)>) {
    let mut vecofroots: Vec<(f64, f64)> = vecofrootst.clone();
    let mut elemlist: Vec<(f64, f64)> = elemlistt.clone();
    let mut vecsremoved: Vec<(f64, f64)> = Vec::new();
    let elemkey = (elem.0 * 1000000.) as i64;
    let fromelem = &converttomolec[&elemkey];
    if (elemlist.contains(&elem)){
        vecsremoved.push(elem);
        elemlist.remove(elemlist.iter().position(|&x| x == elem).unwrap());
    }
    let mut vecelemempty: Vec<(f64, f64)> = Vec::new();
    for elemsub in fromelem {
        if elemlist.contains(elemsub) {
            let output = untreeify(convertfrommolec, converttomolec, Vec::new(), elemlist.clone(), elemsub.clone(), false);
            for removed in output.1 {
                vecsremoved.push(removed);
                elemlist.remove(elemlist.iter().position(|&x| x == removed).unwrap());
            }
            for added in output.0 {
                vecofroots.push(added);
            }
        }
    }
    if(elemlist.len() > 0){
        if maintreemode {
            return untreeify(convertfrommolec, converttomolec, vecofrootst, elemlist.clone(), elemlist[0], true);
        }else{
            return (vecofroots, vecsremoved);
        }
    }else{
        return (vecofroots, vecsremoved);
    }
    todo!();
}
pub fn treeify(
    convertfrommolec: &HashMap<i64, Vec<(f64, f64)>>,
    converttomolec: &HashMap<i64, Vec<(f64, f64)>>,
    vecofrootst: Vec<(f64, f64)>,
    elemlistt: Vec<(f64, f64)>,
    elem: (f64, f64),
    maintreemode: bool,
) -> (Vec<(f64, f64)>, Vec<(f64, f64)>) {
    let mut vecofroots: Vec<(f64, f64)> = vecofrootst.clone();
    let mut elemlist: Vec<(f64, f64)> = elemlistt.clone();
    let mut vecsremoved: Vec<(f64, f64)> = Vec::new();
    let elemkey = (elem.0 * 1000000.) as i64;
    let fromelem = &convertfrommolec[&elemkey];
    if (elemlist.contains(&elem)){
        vecsremoved.push(elem);
        elemlist.remove(elemlist.iter().position(|&x| x == elem).unwrap());
    }
    let mut vecelemempty: Vec<(f64, f64)> = Vec::new();
    for elemsub in fromelem {
        if elemlist.contains(elemsub) {
            let output = treeify(convertfrommolec, converttomolec, Vec::new(), elemlist.clone(), elemsub.clone(), false);
            for removed in output.1 {
                vecsremoved.push(removed);
                elemlist.remove(elemlist.iter().position(|&x| x == removed).unwrap());
            }
            for added in output.0 {
                vecofroots.push(added);
            }
        }
    }
    if(elemlist.len() > 0){
        if maintreemode {
            return treeify(convertfrommolec, converttomolec, vecofrootst, elemlist.clone(), elemlist[0], true);
        }else{
            return (vecofroots, vecsremoved);
        }
    }else{
        return (vecofroots, vecsremoved);
    }
    todo!();
}