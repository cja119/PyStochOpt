# Quickstart
This module is built in rust in order to leverage parallelisation to vastly improve execution speeds. As such it is necessary to install rust on your local machine to use this package. 
### Step 1: Install Rust

1. Open your terminal.
2. Run the following command to install Rust using `rustup` (the Rust toolchain installer):

    ```sh
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
    ```
    Or through anaconda...

    ```sh
    conda install conda-forge::rust
    ```

3. Follow the on-screen instructions to complete the installation.
4. After installation, ensure that Rust is installed correctly by running:

    ```sh
    rustc --version
    ```

    This should display the version of Rust installed.

### Step 2: Install Maturin

1. With Rust installed, you can now install Maturin. Run the following command in your terminal:

    ```sh
    pip install maturin
    ```
    This should display the version of Maturin installed.

For more information, refer to the official documentation:
- [Rust Documentation](https://www.rust-lang.org/learn)
- [Maturin Documentation](https://maturin.rs/)

### Step 3: Clone this repository
```sh
git clone https://github.com/cja119/PyStochOpt.git
```
### Step 4: Build The Module

```sh
maturin develop
```

This should succesfully build the module and store it in your native python location (pay attention to which location it prints when the module is built)

## Example Script

The below exerpt builds a 3 stage, 3 scenario stochastic grid where each stage lasts 168 hours. It then builds two time sets: one with a heirarchy for decisions made every 48 hours and another for an hourly decision, but with a 1 hour delay (useful for modeling difference equations, for example). 

```py
import PyStochOpt

# Building a stochastic grid with 3 stages, each of which lasts 168 hours with 3 stochastic scenarios (ie, three branches per stage).
StochasticGrid = PyStochOpt.StochasticGrid(n_stages = 3, n_scenarios = 3, stage_duration = 168)

# Building a heirarchical grid for a decision made every 48 hours
grid_48_hour = zip(StochasticGrid.get_grid(),StochasticGrid.new_grid(48))

# Building a hourly grid with a 1 hour delay. 
delay_1_hour = zip(StochasticGrid.get_grid(),StochasticGrid.new_grid(1,1))

# Loading a weather dataset for contiguos grid sampling
sampled_dataset = stochastic_grid.add_dataset(file_name='FileName.csv',file_path='/Filepath/')

# Recalling the number of leafnodes corresponding to each branch
leaf_nodes = stochastic_grid.leaf_nodes()
```
