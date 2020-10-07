# clingo-server

Install clingo development version

```sh
conda install -c potassco/label/dev clingo
```

Set path variables to shared clingo library

```sh
export CLINGO_LIBRARY_PATH=/scratch/miniconda/envs/test/lib
export LD_LIBRARY_PATH=/scratch/miniconda/envs/test/lib
```

Start the server with

```sh
cargo +nightly run
```

Test the server with

```sh
python client.py -i queens.lp
```

or

```sh
python client.py -i pigeonhole.lp
```
