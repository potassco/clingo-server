import argparse
import sys
import time
import traceback
import requests
import urllib.parse.urljoin as urljoin

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

        response = requests.get(urljoin(server,'create'))
        if response.status_code == 500:
            print("Solver already running ...")
            print("Shutting down old solver ...")
            response = requests.get(urljoin(server,'close'))
            response = requests.get(urljoin(server,'create'))
            print(response.text)
        response = requests.post(urljoin(server,'loadtheory'), 'clingo-dl',
                                 headers={
                                     "Content-Type": "text/plain; charset=utf-8 "}
                                 )
        if response.status_code == 500:
            raise Exception('Theory clingo-dl not supported')

        with open(args.input, 'rb') as f:
            response = requests.post(urljoin(server,'add'), data=f.read(),
                                     headers={
                                         "Content-Type": "text/plain; charset=utf-8 "}
                                     )
            data = response.text
            print(data)

        response = requests.get(urljoin(server,'ground'))
        data = response.text
        print(data)
        response = requests.get(urljoin(server,'solve'))
        data = response.text
        print(data)

        count = 0
        while True:
            response = requests.get(
                'urljoin(server,'model'), timeout=1)

            if response.status_code == 200:
                json_response = response.json()

                if json_response == 'Running':
                    print("No model yet ... waiting 10 seconds.")
                    time.sleep(10)
                elif json_response == 'Done':
                    print("Search finished, no more models.")
                    break
                else:
                    model = json_response['Model']
                    count += 1
                    print("Model", count, ':')
                    print(model)
                    response = requests.get(urljoin(server,'resume'))
                    print(response.text)
            else:
                print("ServerError")
                print(response.text)
                break

        response = requests.get(urljoin(server,'close'))
        print(response.text)
    except Exception as e:
        print(e)
        traceback.print_exception(*sys.exc_info())
        return 1


if __name__ == '__main__':
    sys.exit(main())
