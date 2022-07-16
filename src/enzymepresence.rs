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
use std::*;

pub fn enzymespresence(
    omsmap: OptimizedMSMap,
    enzymes: Vec<Enzyme>,
    marginoferrorinmass: f64,
) -> OptimizedMSMap {
    let mut xval = 0;
    let mut hashofintensitymaps: HashMap<i64, V2RangeAndIntensityMap> = HashMap::new();
    // List of enzymes that convert to and from a given molecule
    // List of enzymes that can turn a molecule into something new
    let mut convertfrommolec: HashMap<i64, Vec<Enzyme>> = HashMap::new();
    let mut avermolecconsistancy: HashMap<i64, f64> = HashMap::new();
    // List of enzymes that can turn something into a molecule
    let mut converttomolec: HashMap<i64, Vec<Enzyme>> = HashMap::new();
    let templatearrayofintensity: Array2D<f32> = Array2D::filled_with(
        0.,
        omsmap.arrayspec.num_rows(),
        omsmap.arrayspec.num_columns(),
    );
    let mut keylist: Vec<i64> = Vec::new();
    for enzyme in &enzymes {
        let startintmap = V2RangeAndIntensityMap::new(
            templatearrayofintensity.clone(),
            enzyme.activeconversionstartmolecule.mass - marginoferrorinmass,
            enzyme.activeconversionstartmolecule.mass + marginoferrorinmass,
            1.,
            0.,
        );
        let endintmap = V2RangeAndIntensityMap::new(
            templatearrayofintensity.clone(),
            enzyme.activeconversionendmolecule.mass - marginoferrorinmass,
            enzyme.activeconversionendmolecule.mass + marginoferrorinmass,
            1.,
            0.,
        );
        let startkey = (enzyme.activeconversionstartmolecule.mass * 1000000.) as i64;
        let endkey = (enzyme.activeconversionendmolecule.mass * 1000000.) as i64;
        if (!avermolecconsistancy.contains_key(&startkey)) {
            avermolecconsistancy.insert(startkey, 0.);
        }
        if (!avermolecconsistancy.contains_key(&endkey)) {
            avermolecconsistancy.insert(endkey, 0.);

        }
        if (convertfrommolec.contains_key(&startkey)) {
            convertfrommolec
                .get_mut(&startkey)
                .unwrap()
                .push(enzyme.clone());
        } else {
            convertfrommolec.insert(startkey, Vec::new());
            convertfrommolec
                .get_mut(&startkey)
                .unwrap()
                .push(enzyme.clone());
        }
        if (converttomolec.contains_key(&endkey)) {
            converttomolec
                .get_mut(&endkey)
                .unwrap()
                .push(enzyme.clone());
        } else {
            converttomolec.insert(endkey, Vec::new());
            converttomolec
                .get_mut(&endkey)
                .unwrap()
                .push(enzyme.clone());
        }
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
    for enzyme in &enzymes {
        let mut x2val = 0;
        let startkey = (enzyme.activeconversionstartmolecule.mass * 1000000.) as i64;
        let endkey = (enzyme.activeconversionendmolecule.mass * 1000000.) as i64;
        let mut startmap: &V2RangeAndIntensityMap = hashofintensitymaps.get(&startkey).unwrap();
        let mut endmap: &V2RangeAndIntensityMap = hashofintensitymaps.get(&endkey).unwrap();
        let mut totalavr = 1.;
        let mut totalweighting = 1.;
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
                        ratio = intensitystart / intensityend;
                    }
                }
                let weighting = 1.;
                //let weighting = intensitystart * intensityend;
                // if (ratio > 0. && !(ratio == 1.)) {
                //     println!(
                //         "Ratio: {:?}, Weighting: {:?}",
                //         ratio,
                //         weighting
                //     );
                // }
                totalavr = totalavr + (ratio * weighting);
                totalweighting = totalweighting + weighting;
                y2val += 4;
            }
            x2val += 4;
        }
        totalavr = totalavr / totalweighting as f32;
        let startmoleccons = avermolecconsistancy[&startkey];
        let endmoleccons = avermolecconsistancy[&endkey];
        avermolecconsistancy.insert(startkey, startmoleccons.clone() + totalavr as f64);
        avermolecconsistancy.insert(endkey, endmoleccons.clone() + totalavr as f64);
    }
    for key in keylist {
        let consistency = avermolecconsistancy[&key];
        avermolecconsistancy.insert(key, consistency / enzymes.len() as f64);
        println!(
            "AvrRatio: {:?}, for mz ( {:?}, {:?} )",
            avermolecconsistancy[&key], hashofintensitymaps[&key].minmz, hashofintensitymaps[&key].maxmz
        );
    }
    omsmap
}
