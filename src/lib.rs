#![allow(non_snake_case)]
use pyo3::prelude::*;
use rayon::prelude::*;
use pyo3::types::PyTuple;
use csv::Reader;
use rand::Rng;
/// Formats the sum of two numbers as string.


fn read_csv(file_name: &str, file_path: Option<&str>) -> Vec<(usize, f64)> {
    let path = match file_path {
        Some(path) => format!("{}/{}", path, file_name),
        None => file_name.to_string(),
    };
    let mut rdr = Reader::from_path(path).expect("Failed to open CSV file");
    let mut rows: Vec<(usize, f64)> = Vec::new();
    for (i, result) in rdr.records().enumerate() {
        if i == 0 {
            continue; // Skip the first line
        }
        let mut record = result.expect("Failed to read CSV record");
        if record.iter().any(|field| field.contains(' ')) {
            record = csv::StringRecord::from(record.iter().flat_map(|field| field.split_whitespace()).collect::<Vec<&str>>());
        }
        let row: (usize, f64) = (
            record[0].parse().unwrap(),
            record[1].parse().unwrap()
        );
        rows.push(row);
    }
    rows
}

#[pyclass]
struct StochasticGrid {
    n_stages: usize,
    n_scenarios: usize,
    stage_duration: usize,
    grid: Vec<(usize,usize)> ,
}

fn build_grid(n_stages: usize, n_scenarios: usize,stage_duration: usize) ->  Vec<(usize,usize)> {
    let total_time: usize = stage_duration * (n_scenarios.pow(n_stages as u32+ 1) - 1) / (n_scenarios - 1);
    let mut grid: Vec<(usize,usize)>   = vec![(0,0); total_time];

    (0..((n_stages+1) * stage_duration)).into_par_iter().map(|t| {
        let stage: usize = (t as f64 / stage_duration as f64).floor() as usize;
        let scenario: usize = n_scenarios.pow(stage as u32);
        (0..scenario).map(|s| {
            let key: usize = scenario * (t - stage * stage_duration) + s + stage_duration * (scenario -1) / (n_scenarios - 1);
            (key, (s, t))
        }).collect::<Vec<_>>()
    }).flatten().collect::<Vec<_>>().into_iter().for_each(|(key, value)| {
        grid[key] = value;
    });
    return grid;
}   

#[pymethods]
impl StochasticGrid {
    #[new]
    fn new(n_stages: usize, n_scenarios: usize, stage_duration:usize) -> Self {
        let grid:  Vec<(usize,usize)> =  build_grid(n_stages, n_scenarios, stage_duration);
        return StochasticGrid {n_stages, n_scenarios, stage_duration, grid};
    }

    fn get_grid(&self, py: Python<'_>) -> PyResult<Vec<PyObject>>  {
        
        let python_list: Vec<PyObject> = self.grid
            .clone()
            .into_iter()
            .map(|(x, y)| PyTuple::new_bound(py, &[x, y]).to_object(py))
            .collect();
        return  Ok(python_list);
    }

    #[pyo3(signature = (grid_duration, delay=None))]
    fn new_grid(&mut self, grid_duration: usize, delay: Option<usize>) -> Vec<(usize,usize)> {
        let total_time: usize = self.stage_duration * (self.n_scenarios.pow(self.n_stages as u32+ 1) - 1) / (self.n_scenarios - 1);
        let delay: usize = delay.unwrap_or(0);
        let mut new_grid: Vec<(usize,usize)>   = vec![(0,0); total_time];
        (0..((self.n_stages+1) * self.stage_duration)).into_par_iter().map(|t| {
            if t < delay {     
                let grid_t:usize = ((t as f64 / grid_duration as f64).floor() as usize) * grid_duration as usize;
                let grid_stage: usize = (grid_t as f64 / self.stage_duration as f64).floor() as usize;
                let stage: usize = (t as f64 / self.stage_duration as f64).floor() as usize;
                let scenario: usize = self.n_scenarios.pow(stage as u32);  
                (0..scenario).map(|s| {
                    let key: usize = scenario * (t - stage * self.stage_duration) + s + self.stage_duration * (scenario -1) / (self.n_scenarios - 1);
                    let n_stages = self.n_stages;
                    let ratio: usize = n_stages.pow((stage - grid_stage) as u32);
                    (key, ((s as f64/ratio as f64).floor() as usize, 0))
                    }).collect::<Vec<_>>()
                } 
                else {     
                    let grid_t:usize = (((t - delay) as f64 / grid_duration as f64).floor() as usize) * grid_duration as usize;
                    let grid_stage: usize = (grid_t as f64 / self.stage_duration as f64).floor() as usize;
                    let stage: usize = (t  as f64 / self.stage_duration as f64).floor() as usize;
                    let scenario: usize = self.n_scenarios.pow(stage as u32);  
                    (0..scenario).map(|s| {
                        let key: usize = scenario * (t - stage * self.stage_duration) + s + self.stage_duration * (scenario -1) / (self.n_scenarios - 1);
                        let ratio: usize = self.n_stages.pow((stage - grid_stage) as u32);
                        (key, ((s as f64/ratio as f64).floor() as usize, grid_t))
                        }).collect::<Vec<_>>()
                    }
            }).flatten().collect::<Vec<_>>().into_iter().for_each(|(key, value)| {
                new_grid[key] = value;
            }); 
            return new_grid;
        }
        #[pyo3(signature = (file_name, file_path=None))]
        fn add_dataset(&mut self, file_name: &str, file_path: Option<&str>) -> Vec<(usize, f64)> {
            let dataset: Vec<(usize, f64)> = read_csv(file_name, file_path);
            let total_time: usize = self.stage_duration * (self.n_scenarios.pow(self.n_stages as u32+ 1) - 1) / (self.n_scenarios - 1);
            let mut samples: Vec<(usize,f64)>   = vec![(0,0.0); total_time];
            let mut start_points: Vec<usize> = vec![0; self.n_scenarios.pow(self.n_stages as u32)];

            for stage in 0..self.n_stages {
                if stage != 0 {
                    for s in self.n_scenarios.pow((stage - 1) as u32)..self.n_scenarios.pow(stage as u32) {
                        let n_branch: usize = self.n_stages + 1 - (s as f64 + 1.0).log(self.n_scenarios as f64).ceil() as usize;
                        let n_samp: usize = (n_branch * (n_branch + 1) * (n_branch + 2) / 6) * self.stage_duration;
                        let startpoint = rand::thread_rng().gen_range(0..dataset.len() - n_samp);
                        start_points[s] = startpoint;
                }}
                else {
                    let s = stage;
                    let n_branch: usize = self.n_stages + 1;
                    let n_samp: usize = (n_branch * (n_branch + 1) * (n_branch + 2) / 6) * self.stage_duration;
                    let startpoint = rand::thread_rng().gen_range(0..dataset.len() - n_samp);
                    start_points[s] = startpoint;
                }
            }

            
            for s in 0..self.n_scenarios.pow(self.n_stages as u32) {
                for t in self.stage_duration * (s as f64 + 1.0).log(self.n_scenarios as f64).ceil() as usize..self.stage_duration * (self.n_stages + 1) as usize {
                    let stage: usize = (t as f64 / self.stage_duration as f64).floor() as usize;
                    let scenario: usize = self.n_scenarios.pow(stage as u32);  
                    let key: usize = scenario * (t - stage * self.stage_duration) + s + self.stage_duration * (scenario -1) / (self.n_scenarios - 1);
                    
                    samples[key] = (s, dataset[start_points[s] + t - self.stage_duration * (s as f64 + 1.0).log(self.n_scenarios as f64).ceil()  as usize].1);
                }
            }

            return samples;
        }
}

/// A Python module implemented in Rust.
#[pymodule]
fn PyStochOpt(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<StochasticGrid>()?;
    Ok(())
}
