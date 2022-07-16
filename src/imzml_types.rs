use array2d::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct UnoptimizedMSMap {
    pub vecspectrum: Vec<Spectrum>,
    pub xsize: i64,
    pub ysize: i64,
}
#[derive(Debug, Clone)]
pub struct OptimizedMSMap {
    pub arrayspec: Array2D<OSpectrum>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizedMSMapTransitionstate {
    pub arrayspec: Vec<OSpectrum>,
    pub sizex: usize,
    pub sizey: usize,
}
#[derive(Debug, Clone, PartialEq)]
pub struct V2RangeAndIntensityMap {
    pub arrayintensity: Array2D<f32>,
    pub minmz: f64,
    pub maxmz: f64,
    pub highestintensity: f32,
    pub avervalnotrelativetointensity: f32,
}
impl V2RangeAndIntensityMap {
    /// Construct a new XZVert.
    pub fn new(
        arrayintensity: Array2D<f32>,
        minmz: f64,
        maxmz: f64,
        highestintensity: f32,
        avervalnotrelativetointensity: f32,
    ) -> Self {
        V2RangeAndIntensityMap {
            arrayintensity,
            minmz,
            maxmz,
            highestintensity,
            avervalnotrelativetointensity,
        }
    }
}
#[derive(Debug, Clone, PartialEq)]
pub struct RangeAndIntensityMap {
    pub arrayintensity: Array2D<f32>,
    pub minmz: f64,
    pub maxmz: f64,
    pub highestintensity: f32,
}
impl RangeAndIntensityMap {
    /// Construct a new XZVert.
    pub fn new(
        arrayintensity: Array2D<f32>,
        minmz: f64,
        maxmz: f64,
        highestintensity: f32,
    ) -> Self {
        RangeAndIntensityMap {
            arrayintensity,
            minmz,
            maxmz,
            highestintensity,
        }
    }
}
#[derive(Debug, Clone)]
pub struct RatioRangeAndRGBImage {
    pub imagem: image::RgbImage,
    pub minmz1: f64,
    pub maxmz1: f64,
    pub minmz2: f64,
    pub maxmz2: f64,
    pub highestintensity1: f32,
    pub highestintensity2: f32,
}
#[derive(Debug, Clone)]
pub struct CircularCorrilationImage {
    pub ratioimg: RatioRangeAndRGBImage,
    pub corcirc1: RatioRangeAndRGBImage,
    pub corcirc2: RatioRangeAndRGBImage,
    pub corcirc3: RatioRangeAndRGBImage,
    pub corcirc4: RatioRangeAndRGBImage,
    pub corcirc5: RatioRangeAndRGBImage,
    pub corcirc6: RatioRangeAndRGBImage,
    pub corrilationscore1: f64,
    pub corrilationscore2: f64,
    pub corrilationscore3: f64,
    pub corrilationscore4: f64,
    pub corrilationscore5: f64,
    pub corrilationscore6: f64,
    pub avrratio1over2: f64,
    pub notetouser: String,
}
impl CircularCorrilationImage {
    /// Construct a new XZVert.
    pub fn new(
        ratioimg: RatioRangeAndRGBImage,
        corcirc1: RatioRangeAndRGBImage,
        corcirc2: RatioRangeAndRGBImage,
        corcirc3: RatioRangeAndRGBImage,
        corcirc4: RatioRangeAndRGBImage,
        corcirc5: RatioRangeAndRGBImage,
        corcirc6: RatioRangeAndRGBImage,
        corrilationscore1: f64,
        corrilationscore2: f64,
        corrilationscore3: f64,
        corrilationscore4: f64,
        corrilationscore5: f64,
        corrilationscore6: f64,
        avrratio1over2: f64,
        notetouser: String,
    ) -> Self {
        CircularCorrilationImage {
            ratioimg,
            corcirc1,
            corcirc2,
            corcirc3,
            corcirc4,
            corcirc5,
            corcirc6,
            corrilationscore1,
            corrilationscore2,
            corrilationscore3,
            corrilationscore4,
            corrilationscore5,
            corrilationscore6,
            avrratio1over2,
            notetouser,
        }
    }
}
#[derive(Debug, Clone)]
pub struct CorrilationImage {
    pub ratioimg: RatioRangeAndRGBImage,
    pub howwellitcorrilates: RatioRangeAndRGBImage,
    pub corrilationscore: f64,
    pub avrratio1over2: f64,
    pub notetouser: String,
}
impl CorrilationImage {
    /// Construct a new XZVert.
    pub fn new(
        ratioimg: RatioRangeAndRGBImage,
        howwellitcorrilates: RatioRangeAndRGBImage,
        corrilationscore: f64,
        avrratio1over2: f64,
        notetouser: String,
    ) -> Self {
        CorrilationImage {
            ratioimg,
            howwellitcorrilates,
            corrilationscore,
            avrratio1over2,
            notetouser,
        }
    }
}
impl RatioRangeAndRGBImage {
    /// Construct a new XZVert.
    pub fn new(
        imagem: image::RgbImage,
        minmz1: f64,
        maxmz1: f64,
        minmz2: f64,
        maxmz2: f64,
        highestintensity1: f32,
        highestintensity2: f32,
    ) -> Self {
        RatioRangeAndRGBImage {
            imagem,
            minmz1,
            maxmz1,
            minmz2,
            maxmz2,
            highestintensity1,
            highestintensity2,
        }
    }
}
#[derive(Debug, Clone)]
pub struct RangeAndRGBImage {
    pub imagem: image::RgbImage,
    pub minmz: f64,
    pub maxmz: f64,
    pub highestintensity: f32,
}
impl RangeAndRGBImage {
    /// Construct a new XZVert.
    pub fn new(imagem: image::RgbImage, minmz: f64, maxmz: f64, highestintensity: f32) -> Self {
        RangeAndRGBImage {
            imagem,
            minmz,
            maxmz,
            highestintensity,
        }
    }
}

impl OptimizedMSMap {
    /// Construct a new XZVert.
    pub fn new(x: usize, y: usize) -> Self {
        OptimizedMSMap {
            arrayspec: Array2D::fill_with(OSpectrum::new(0., Vec::new(), 0., 0.), x, y),
        }
    }
    pub fn set(&self, oms: OptimizedMSMap)  -> Self {
        oms
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OSpectrum {
    pub highestintensity: f32,
    pub peeks: Vec<OSpecPeeks>,
    pub min: f32,
    pub max: f32,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Enzyme {
    pub name: String,
    pub activeconversionstartmolecule: Molecule,
    pub activeconversionendmolecule: Molecule,
}
impl Enzyme {
    /// Construct a new XZVert.
    pub fn new(
        name: String,
        activeconversionstartmolecule: Molecule,
        activeconversionendmolecule: Molecule,
    ) -> Self {
        Enzyme {
            name,
            activeconversionstartmolecule,
            activeconversionendmolecule,
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Molecule {
    pub name: String,
    pub mass: f64,
}
impl Molecule {
    /// Construct a new XZVert.
    pub fn new(
        name: String,
        mass: f64,
    ) -> Self {
        Molecule {
            name,
            mass
        }
    }
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct OSpecPeeks {
    pub relativemztominandmax: u32,
    pub intensityrelativetomax: u16,
    pub slopepermzover10: u16,
}
#[derive(Debug, Clone, Copy)]
pub struct USpecPeeks {
    pub mz: f64,
    pub intensity: f32,
    pub slopepermz: f64,
}
impl OSpecPeeks {
    /// Construct a new XZVert.
    pub fn new(
        relativemztominandmax: u32,
        intensityrelativetomax: u16,
        slopepermzover10: u16,
    ) -> Self {
        OSpecPeeks {
            relativemztominandmax,
            intensityrelativetomax,
            slopepermzover10,
        }
    }
}
impl USpecPeeks {
    /// Construct a new XZVert.
    pub fn new(mz: f64, intensity: f32, slopepermz: f64) -> Self {
        USpecPeeks {
            mz,
            intensity,
            slopepermz,
        }
    }
}

impl OSpectrum {
    /// Construct a new XZVert.
    pub fn new(highestintensity: f32, peeks: Vec<OSpecPeeks>, min: f32, max: f32) -> Self {
        OSpectrum {
            highestintensity,
            peeks,
            min,
            max,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Spectrum {
    pub points: Vec<Mzplusintensity>,
    pub x: i64,
    pub y: i64,
}

impl Spectrum {
    /// Construct a new XZVert.
    pub fn new(points: Vec<Mzplusintensity>, x: i64, y: i64) -> Self {
        Spectrum { points, x, y }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Mzplusintensity {
    pub mz: f64,
    pub intensity: f32,
}
impl Mzplusintensity {
    /// Construct a new XZVert.
    pub fn new(mz: f64, intensity: f32) -> Self {
        Mzplusintensity { mz, intensity }
    }
}
impl UnoptimizedMSMap {
    /// Construct a new XZVert.
    pub fn new(vecspectrum: Vec<Spectrum>, xsize: i64, ysize: i64) -> Self {
        UnoptimizedMSMap {
            vecspectrum: vecspectrum,
            xsize: xsize,
            ysize: ysize,
        }
    }
    pub fn add_spectrum(self: &Self, spec: Spectrum) -> UnoptimizedMSMap {
        let mut selfvec: Vec<Spectrum> = self.vecspectrum.clone();
        selfvec.push(spec);
        UnoptimizedMSMap {
            vecspectrum: selfvec,
            xsize: self.xsize,
            ysize: self.ysize,
        }
    }
}
