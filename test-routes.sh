#!/bin/bash
set -eo pipefail

# Import common functions
DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
# shellcheck source=../common.sh
source "$DIR/common.sh"

USAGE="Usage: $0 [OPTIONS]

Test API routes.
Port number can be set with env variable: PORT=80 $0

OPTIONS: All options are optional
    -h | --help
        Display these instructions.

    -p | --port [NUMBER]
        Specify port number to use. Default is 3000.

    -v | --verbose
        Display commands being executed."

while [ $# -gt 0 ]; do
    case "$1" in
        -h | --help)
            print_usage_and_exit
            ;;
        -p | --port)
            PORT=$2
            shift
            ;;
        -v | --verbose)
            set -x
            ;;
    esac
    shift
done

PORT=${PORT:-3000}

get() {
    local url="$1"
    print_magenta "GET: $1"
    response=$(curl -s -w "%{http_code}" -o response.json "$url")
    print_response "$response"
}

post() {
    local url="$1"
    local data="$2"
    print_cyan "POST: $1 $2"
    response=$(curl -s -X POST -H "Content-Type: application/json" -d "$data" -w "%{http_code}" -o response.json "$url")
    print_response "$response"
}

delete() {
    local url="$1"
    print_red "DELETE: $1"
    response=$(curl -s -X DELETE -w "%{http_code}" -o response.json "$url")
    print_response "$response"
}

print_response() {
    local response="$1"
    if echo "$response" | grep -q '^2'; then
        echo "Status code: $(green "$response")"
    elif echo "$response" | grep -qE '^[45]'; then
        echo "Status code: $(red "$response")"
    else
        echo "Status code: $response"
    fi
    output=$(jq --color-output < response.json)
    if [ "$(echo "$output" | wc -l)" -gt 1 ]; then
        echo "Response:"
        echo "$output"
    else
        echo "Response: $output"
    fi
    rm response.json
}

if ! curl -s -o /dev/null -w "%{http_code}" "http://127.0.0.1:$PORT" | grep -q '^2'; then
    print_error_and_exit "Failed to call API, is it running?"
fi

get "http://127.0.0.1:$PORT"
get "http://127.0.0.1:$PORT/version"
get "http://127.0.0.1:$PORT/list_items"
post "http://127.0.0.1:$PORT/items" '{"name":"esgrove"}'
get "http://127.0.0.1:$PORT/item?name=esgrove"
post "http://127.0.0.1:$PORT/items" '{"name":"esgrove"}'
post "http://127.0.0.1:$PORT/items" '{"name":"five","id":5555}'
post "http://127.0.0.1:$PORT/items" '{"name":"error","id":1}'
get "http://127.0.0.1:$PORT/item?name=pizzalover9000"

for name in pizzalover9000 akseli swanson; do
    post "http://127.0.0.1:$PORT/items" "{\"name\":\"$name\"}"
done

get "http://127.0.0.1:$PORT/list_items"

# Trying to use GET with admin routes results in 405 "Method Not Allowed"
get "http://127.0.0.1:$PORT/admin/remove/pizzalover"

delete "http://127.0.0.1:$PORT/admin/remove/pizzalover"
delete "http://127.0.0.1:$PORT/admin/remove/pizzalover9000"

get "http://127.0.0.1:$PORT/list_items"

delete "http://127.0.0.1:$PORT/admin/clear_items"

get "http://127.0.0.1:$PORT/list_items"
