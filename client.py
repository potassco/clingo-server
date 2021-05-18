import argparse
import sys
import io
import time
import traceback
import requests
import json

server = 'http://localhost:8000/'
conf_fail = '{\
  "tester": {\
    "share": ""\
  }\
}'
conf = '{\
  "tester": {\
    "solver": [],\
    "configuration": "auto",\
    "share": "auto",\
    "learn_explicit": "0",\
    "sat_prepro": "no"\
  },\
  "solve": {\
    "solve_limit": "umax,umax",\
    "parallel_mode": "1,compete",\
    "global_restarts": "no",\
    "distribute": "conflict,global,4,4194303",\
    "integrate": "gp,1024,all",\
    "enum_mode": "auto",\
    "project": "no",\
    "models": "0",\
    "opt_mode": "opt"\
  },\
  "asp": {\
    "trans_ext": "dynamic",\
    "eq": "3",\
    "backprop": "0",\
    "supp_models": "0",\
    "no_ufs_check": "0",\
    "no_gamma": "0",\
    "eq_dfs": "0",\
    "dlp_old_map": "0"\
  },\
  "solver": [],\
  "configuration": "auto",\
  "share": "auto",\
  "learn_explicit": "0",\
  "sat_prepro": "no",\
  "stats": "0",\
  "parse_ext": "false",\
  "parse_maxsat": "false"\
}'


def main():
    try:
        parser = argparse.ArgumentParser(
            description="Test the clingo server")
        parser.add_argument('-i', '--input', required=True,
                            help='logic program')
        parser.add_argument(
            '-c', '--conf',  action='store_true', required=False,
            help='set and get configuration')
        parser.add_argument(
            '-s', '--stats',  action='store_true', required=False,
            help='get statistics')

        parser.add_argument('--assume',  action='store_true', required=False,
                            help='assume queen(3,1) to be true')
        args = parser.parse_args()

        response = requests.get(server)
        print(response.text)

        # create solver
        response = requests.get(server+'create')
        if response.status_code == 500:
            print("Solver already running ...")
            print("Shutting down old solver ...")
            response = requests.get(server+'close')
            response = requests.get(server+'create')
            print(response.text)

        # register theory
        response = requests.get(server+'register_dl_theory')
        print(response.text)

        # add logic program
        with open(args.input, 'rb') as f:
            response = requests.post(server+'add', data=f.read(),
                                     headers={
                "Content-Type": "text/plain; charset=utf-8 "}
            )
            print(response.text)

        # set configuration
        if args.conf:
            response = requests.post(server+'set_configuration', data=io.StringIO(conf).read(),
                                     headers={
                "Content-Type": "application/json; charset=utf-8 "}
            )
            print(response.text)

            # get configuration
            response = requests.get(server+'configuration')
            dictionary = response.json()
            json_formatted_str = json.dumps(dictionary, indent=2)
            print("Configuration:", json_formatted_str)

        # ground
        response = requests.get(server+'ground')
        print(response.text)

        # solve with assumptions
        if args.assume:
            assumptions = '[["queen(3,1)",true]]'  # works with queens.lp
        else:
            assumptions = '[]'
        response = requests.post(server+'solve_with_assumptions', data=io.StringIO(assumptions).read(),
                                 headers={
            "Content-Type": "application/json; charset=utf-8 "}
        )
        print(response.text)

        # poll for models
        count = 0
        while True:
            response = requests.get(
                server+'model', timeout=1)

            if response.status_code == 200:
                json_response = response.json()

                if json_response == 'Running':
                    print("No model yet ... waiting 10 seconds.")
                    time.sleep(10)
                elif json_response == 'Done':
                    print("Search finished, no more models.")
                    break
                elif 'Model' in json_response:
                    model = json_response['Model']
                    count += 1
                    print("Model", count, ':')
                    print(bytes(model).decode("utf-8"))
                    response = requests.get(server+'resume')
                    print(response.text)
                else:
                    print("Error unexpected respose to model/ request")
                    print(json_response)
                    exit()
            else:
                print("ServerError")
                print(response.text)
                break

        response = requests.get(server+'close')
        print(response.text)

        # get statistics
        if args.stats:
            response = requests.get(server+'statistics')
            dictionary = response.json()
            json_formatted_str = json.dumps(dictionary, indent=2)
            print("Statistics:", json_formatted_str)

    except Exception as e:
        print(e)
        traceback.print_exception(*sys.exc_info())
        return 1


if __name__ == '__main__':
    sys.exit(main())
