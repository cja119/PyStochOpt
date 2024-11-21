#![allow(non_snake_case)]
use pyo3::{prelude::*, types::PyDict};
use rayon::prelude::*;
use pyo3::types::PyTuple;
use csv::Reader;
use rand::{Rng, SeedableRng};


fn read_csv(file_name: &str, file_path: Option<&str>) -> Vec<(usize, f64)> {
    let path = match file_path {
        Some(path) => format!("{}/{}", path, file_name),
        None => file_name.to_string(),
    };
    let mut rdr = Reader::from_path(path).expect("Failed to open CSV file");
    let mut rows: Vec<(usize, f64)> = Vec::new();
    for (i, result) in rdr.records().enumerate() {
        if i == 0 {
            continue; 
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
    grid: Vec<(usize,usize)>,
    seed: u64,
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
/// Implementation of the `StochasticGrid` struct.
///
/// # Methods
///
/// - `new(n_stages: usize, n_scenarios: usize, stage_duration: usize) -> Self`
///   - Creates a new instance of `StochasticGrid`.
///   - Parameters:
///     - `n_stages`: Number of stages.
///     - `n_scenarios`: Number of scenarios.
///     - `stage_duration`: Duration of each stage.
///   - Returns: A new `StochasticGrid` instance.
///
/// - `get_grid(&self, py: Python<'_>) -> PyResult<Vec<PyObject>>`
///   - Retrieves the grid as a list of Python objects.
///   - Parameters:
///     - `py`: Python interpreter instance.
///   - Returns: A `PyResult` containing a vector of Python objects representing the grid.
///
/// - `new_grid(&mut self, grid_duration: usize, delay: Option<usize>) -> Vec<(usize, usize)>`
///   - Generates a new grid based on the provided duration and optional delay.
///   - Parameters:
///     - `grid_duration`: Duration of the grid.
///     - `delay`: Optional delay before starting the grid generation.
///   - Returns: A vector of tuples representing the new grid.
///
/// - `add_dataset(&mut self, file_name: &str, file_path: Option<&str>) -> Vec<(usize, f64)>`
///   - Adds a dataset to the grid from a CSV file.
///   - Parameters:
///     - `file_name`: Name of the CSV file.
///     - `file_path`: Optional path to the CSV file.
///   - Returns: A vector of tuples representing the dataset.
impl StochasticGrid {
    #[new]
    #[pyo3(signature = (n_stages, n_scenarios, stage_duration, seed=None))]
    fn new(n_stages: usize, n_scenarios: usize, stage_duration:usize, seed:Option<u64> ) -> Self {
        let seed: u64 = seed.unwrap_or(rand::thread_rng().gen());
        let grid:  Vec<(usize,usize)> =  build_grid(n_stages, n_scenarios, stage_duration);

        
        return StochasticGrid {n_stages, n_scenarios, stage_duration, grid, seed};
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
                    let ratio: usize = self.n_scenarios.pow((stage - grid_stage) as u32);
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
                        let ratio: usize = self.n_scenarios.pow((stage - grid_stage) as u32);
                        (key, ((s as f64/ratio as f64).floor() as usize, grid_t))
                        }).collect::<Vec<_>>()
                    }
            }).flatten().collect::<Vec<_>>().into_iter().for_each(|(key, value)| {
                new_grid[key] = value;
            }); 
            return new_grid;
        }

        #[pyo3(signature = (file_name, file_path=None))]
        fn add_dataset(&mut self, file_name: &str, file_path: Option<&str>) -> PyResult<Py<PyDict>> {
            let dataset: Vec<(usize, f64)> = read_csv(file_name, file_path);
            let total_time: usize = self.stage_duration * (self.n_scenarios.pow(self.n_stages as u32+ 1) - 1) / (self.n_scenarios - 1);
            let mut samples: Vec<(usize,usize,f64)>   = vec![(0,0,0.0); total_time];
            let mut start_points: Vec<usize> = vec![0; self.n_scenarios.pow(self.n_stages as u32)];

            for stage in 0..self.n_stages + 1 {
            if stage != 0 {
                for s in self.n_scenarios.pow((stage - 1) as u32)..self.n_scenarios.pow(stage as u32) {
                let n_branch: usize = self.n_stages + 1 - (s as f64 + 1.0).log(self.n_scenarios as f64).ceil() as usize;
                let n_samp: usize = (n_branch * (n_branch + 1) * (n_branch + 2) / 6) * self.stage_duration;
                let mut rng = rand::rngs::StdRng::seed_from_u64(self.seed);
                let startpoint = rng.gen_range(0..dataset.len() - n_samp);
                start_points[s] = startpoint;
            }}
            else {
                let s = stage;
                let n_branch: usize = self.n_stages + 1;
                let n_samp: usize = (n_branch * (n_branch + 1) * (n_branch + 2) / 6) * self.stage_duration;
                let mut rng = rand::rngs::StdRng::seed_from_u64(self.seed);
                let startpoint = rng.gen_range(0..dataset.len() - n_samp);
                start_points[s] = startpoint;
            }
            }

            (0..self.n_scenarios.pow(self.n_stages as u32)).into_par_iter().map(|s| {
            (self.stage_duration * (s as f64 + 1.0).log(self.n_scenarios as f64).ceil() as usize..self.stage_duration * (self.n_stages + 1) as usize).map(|t| {
                let stage: usize = (t as f64 / self.stage_duration as f64).floor() as usize;
                let scenario: usize = self.n_scenarios.pow(stage as u32);  
                let key: usize = scenario * (t - stage * self.stage_duration) + s + self.stage_duration * (scenario -1) / (self.n_scenarios - 1);
                
                (key, (s,t, dataset[start_points[s] + t - self.stage_duration * (s as f64 + 1.0).log(self.n_scenarios as f64).ceil()  as usize].1))
            }).collect::<Vec<_>>()
            }).flatten().collect::<Vec<_>>().into_iter().for_each(|(key, value)| {
            samples[key] = value;
            }); 
            
            Python::with_gil(|py| {
            let py_dict = PyDict::new_bound(py);
            for (_key, (s, t, value)) in samples.iter().enumerate() {
            py_dict.set_item((s, t), value).unwrap();
            }
            
            Ok(py_dict.into())
            })

        }
        #[pyo3(signature = (grid))]
        fn remove_duplicates(&mut self, grid: Vec<(usize,usize)>) -> Vec<(usize,usize)> {
            let mut seen: Vec<(usize,usize)> = Vec::new();
            for (_i, (s, t)) in grid.iter().enumerate() {
                if !seen.contains(&(*s, *t)) {
                    seen.push((*s,*t));
                }
            }
            seen
        }

        fn leaf_nodes(&self, py: Python<'_>) -> PyResult<Py<PyDict>> {
            let py_dict = PyDict::new_bound(py);
            let mut seen: Vec<(usize,usize,usize)> = Vec::new();
            for (_i, (s, t)) in self.grid.iter().enumerate() {
                let value = self.n_scenarios.pow(self.n_stages as u32 - (*t as f64 / self.stage_duration as f64).floor() as u32);
                if !seen.contains(&(*s, *t,value)) {
                    seen.push((*s,*t,value));
                }}
            for (_key, (s, t, value)) in seen.iter().enumerate() {
                py_dict.set_item((s, t), value).unwrap();
            }
            

        Ok(py_dict.into())
    }}


#[pymodule]
fn PyStochOpt(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<StochasticGrid>()?;
    Ok(())
}
