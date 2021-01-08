# clingo-server

Set paths to clingo library libclingo.so and clingo-dl libclingo-dl.so

```sh
export CLINGO_LIBRARY_PATH=/scratch/miniconda3/envs/clingo/lib
export CLINGO_DL_LIBRARY_PATH=/clingo-dl/lib
export LD_LIBRARY_PATH=/scratch/miniconda3/envs/clingo/lib:/clingo-dl/lib
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
