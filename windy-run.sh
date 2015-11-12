#!/bin/bash

set -e

pushd "`dirname "$0"`" > /dev/null

function launch-agent {
    local suffix=$1
    local outdir=$2
    echo "Launching agent${suffix}..."
    awk_script='
BEGIN {
    n  = 0
    p  = 0
    b1 = 0
    b2 = 0
    b3 = 0
}
{
    n += 1
    b0 = $5-p
    ravg = (b0+b1+b2+b3)/(n>3 ? 4 : n)
    print n, $5, b0, ravg
    b3 = b2
    b2 = b1
    b1 = b0
    p = $5
}'
    ./windy-agent/target/release/windy-agent$suffix | awk "$awk_script" > "${outdir}/out${suffix}.dat"
}

function launch-all {
    local n=$1 # 4 or 8
    local d=$2 # out.dat dir
    agent_pids=""
    launch-agent "${n}-simple" "$d" &
    agent_pids="$agent_pids $!"

    launch-agent "${n}-r1goal" "$d" &
    agent_pids="$agent_pids $!"

    launch-agent "${n}-rHgoal" "$d" &
    agent_pids="$agent_pids $!"

    launch-agent "${n}-noidle" "$d" &
    agent_pids="$agent_pids $!"

    wait $agent_pids
}

mkdir -p static stochastic

echo "Launching static windy grid..."
./windy-grid/target/release/windy-grid-static < windy-grid/input.json > /dev/null &
grid_pid=$!

sleep 1

launch-all 4 static
launch-all 8 static

echo "Killing windy grid process..."
kill $grid_pid

echo "Launching stochastic windy grid..."
./windy-grid/target/release/windy-grid-stochastic < windy-grid/input.json > /dev/null &
grid_pid=$!

sleep 1

launch-all 4 stochastic
launch-all 8 stochastic

echo "Killing windy grid process..."
kill $grid_pid

echo $'\nGenerating plots...'
gnuplot -e "outdir='static'" windy-plot.gpi
gnuplot -e "outdir='stochastic'" windy-plot.gpi

popd > /dev/null

echo "Bye!"
