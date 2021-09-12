function smoothing () {
    return Math.max(1, +document.querySelector("#smoothing").value);
}

let all_names = [""];
let id_to_item = {};
let name_to_item = {};

function item() {
    let written = document.querySelector("#item").value;
    let closest = stringSimilarity.findBestMatch(written, all_names);
    closest = closest && closest.bestMatch && name_to_item[closest.bestMatch.target];
    console.log(closest);
    if (closest) {
        document.querySelector("#picked").textContent = closest.name;
        return closest.id;
    } else {
        return 2;
    }
}

function register_mapping(mapping) {
    for (let item of mapping) {
        id_to_item[item.id] = item;
        name_to_item[item.name.toLowerCase()] = item;
        all_names.push(item.name.toLowerCase());
    }
}

function smooth(data) {
    var how_much = smoothing();
    let queue_low = []; 
    let queue_high = []; 

    return data.map(function ([time, low, high]){
        queue_low.push(low);
        queue_high.push(high);
        if (queue_low.length > how_much) {queue_low.shift()}
        if (queue_high.length > how_much) {queue_high.shift()}
        let low_avg = queue_low.reduce((a, b) => a + b, 0) / queue_low.length;
        let high_avg = queue_high.reduce((a, b) => a + b, 0) /queue_high.length;
        return [time, low_avg, high_avg];
    });
}

function process(data) {
    let lowest = Infinity;
    let highest = -Infinity;
    data.sort(([a], [b]) => a - b);
    data = smooth(data);
    data = data.map(function([time, low, high]) {
        lowest = Math.min(lowest, low);
        highest = Math.max(highest, high);
        return [new Date(time * 1000), high-low, low];
    });

    let padding = (highest - lowest) / 2;
    lowest = Math.max(lowest - padding, 0);
    highest = highest + padding;
    return {
        data,
        lowest,
        highest
    };
}

function load(element, d) {
    let { lowest, highest, data } = process(d);

    let opts = {
        labels: ["x", "delta", "low"],
        stackedGraph: true,
        valueRange: [lowest, highest],
        stepPlot: smoothing() === 1,
    };

    graph = new Dygraph(element, data, opts);

    var prev = null

    function refresh () {
        fetch(`http://159.65.40.110:8080/${item()}`)
            .then(response => response.json())
            .then(smooth)
            .then(process)
            .then(function({ lowest, highest, data }) {
                if (JSON.stringify(prev) === JSON.stringify(data)) {
                    return;
                }
                prev = data;

                graph.updateOptions({
                    file: data, 
                    valueRange: [lowest, highest],
                    stepPlot: smoothing() === 1,
                });
            });
    }

    document.querySelector("#item").onkeypress = refresh;
    setInterval(refresh, 1000);
}

fetch(`http://159.65.40.110:8080/${item()}`)
    .then(response => response.json())
    .then(x => load(document.querySelector("#graph"), x));

fetch(`https://prices.runescape.wiki/api/v1/osrs/mapping`)
    .then(response => response.json())
    .then(mapping => register_mapping(mapping));