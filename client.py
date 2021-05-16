import argparse
import sys
import time
import traceback
import requests
import json

server = 'http://localhost:8000/'


def main():
    try:
        parser = argparse.ArgumentParser(
            description="Test the clingo server")
        parser.add_argument('-i', '--input',
                            help='logic program', required=True)
        args = parser.parse_args()

        response = requests.get(server)
        data = response.text
        print(data)

        response = requests.get(server+'create')
        if response.status_code == 500:
            print("Solver already running ...")
            print("Shutting down old solver ...")
            response = requests.get(server+'close')
            response = requests.get(server+'create')
            print(response.text)

        response = requests.get(server+'register_dl_theory')
        data = response.text
        print(data)

        with open(args.input, 'rb') as f:
            response = requests.post(server+'add', data=f.read(),
                                     headers={
                                         "Content-Type": "text/plain; charset=utf-8 "}
                                     )
            data = response.text
            print(data)

        response = requests.get(server+'ground')
        data = response.text
        print(data)

        response = requests.get(server+'solve')
        data = response.text
        print(data)

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

        response = requests.get(server+'statistics')
        dictionary = response.json()
        json_formatted_str = json.dumps(dictionary, indent=2)
        print("Statistics:", json_formatted_str)

        response = requests.get(server+'configuration')
        dictionary = response.json()
        json_formatted_str = json.dumps(dictionary, indent=2)
        print("Configuration:", json_formatted_str)

    except Exception as e:
        print(e)
        traceback.print_exception(*sys.exc_info())
        return 1


if __name__ == '__main__':
    sys.exit(main())
