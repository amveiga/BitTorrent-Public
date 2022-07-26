/* eslint-disable react-hooks/exhaustive-deps */
import axios from "axios";
import logo from "./logo.svg";
import btLogo from "./BitTorrent_logo.svg";
import crab from "./Crab.svg";
import "./App.css";
import useAsyncState from "./hooks/use-async-state";
import { useEffect, useMemo, useState } from "react";
import { HiPlus as PlusIcon } from "react-icons/hi";
import moment from "moment";
import { Select, MenuItem, InputLabel, FormControl } from "@mui/material";
import styles from "./styles.module.scss";
import { Bar } from "react-chartjs-2";
import Chart from "chart.js/auto";
import { MOCK_TORRENT } from "./constants";

const headers = [
  "peer id",
  "ip",
  "port",
  "event",
  "uploaded",
  "left",
  "connected"
];

const filteringOptions = [
  {
    label: "Todos",
    value: null,
    graph: null
  },
  {
    label: "Últimos 15 minutos",
    value: 900,
    graph: {
      displayAs: ["Minutos"]
    }
  },
  {
    label: "Última hora",
    value: 3600,
    graph: {
      displayAs: ["Minutos"]
    }
  },
  {
    label: "Últimas 5 horas",
    value: 18000,
    graph: {
      displayAs: ["Minutos", "Horas"]
    }
  },
  {
    label: "Último día",
    value: 86400,
    graph: {
      displayAs: ["Horas"]
    }
  },
  {
    label: "Últimos 3 días",
    value: 259200,
    graph: {
      displayAs: ["Horas"]
    }
  }
];

const renderColumn = {
  created_at: (value) =>
    moment
      .duration(
        moment(new Date()).diff(moment(new Date()).subtract(value, "seconds"))
      )
      .humanize()
};

const App = () => {
  const [selectedTorrent, setSelectedTorrent] = useState(null);
  const [timeFilter, setTimeFilter] = useState(undefined);
  const [graphSectioning, setGraphSectioning] = useState(null);

  const handleTimeFilterChange = ({ target }) => {
    setTimeFilter(target.value);
    setGraphSectioning(null);
  };

  const { data, refresh, inProgress } = useAsyncState({}, () =>
    axios
      .get("http://127.0.0.1:8080/stats", {
        method: "GET",
        headers: { "Access-Control-Allow-Origin": "*" }
      })
      .then((response) => ({ ...response.data, ...MOCK_TORRENT }))
      .catch(() => ({}))
  );

  useEffect(() => {
    refresh();
    window.setInterval(refresh, 5000);
  }, []);

  /*
  
  {
    [info_hash]: {
      peer_list: {
        peers: { id, ip, port }[]
      }                            ->  { id, ip, port }[]
    },
    [info_hash]: {
      peer_list: {
        peers: { id, ip, port }[]
      }
    }
  }

  [] {id, ip, port}[]
  
  
  */

  const flattenedData = Object.keys(data)
    .map((key) =>
      (data[key].peer_list.peers || []).filter(({ created_at }) =>
        timeFilter ? timeFilter >= created_at : true
      )
    )
    .flat()
    .map(({ created_at }) => created_at);

  const filteringOption = filteringOptions.find(
    ({ value }) => value === timeFilter
  );

  const hourData = useMemo(() => {
    if (
      !timeFilter ||
      !timeFilter / 3600 > 0 ||
      graphSectioning === "Minutos"
    ) {
      return [];
    }

    const data = new Array(Math.round(timeFilter / 3600)).fill(0);

    flattenedData.forEach((peerCreatedAt) => {
      const hr = Math.floor(peerCreatedAt / 3600);

      data[hr] += 1;
    });

    return data.reverse();
  }, [graphSectioning, timeFilter]);

  const minuteData = useMemo(() => {
    if (!timeFilter || !timeFilter / 60 > 0 || graphSectioning === "Horas") {
      return [];
    }

    const data = new Array(Math.round(timeFilter / 60)).fill(0);

    flattenedData.forEach((peerCreatedAt) => {
      const min = Math.ceil(peerCreatedAt / 60);

      data[min] += 1;
    });

    return data.reverse();
  }, [graphSectioning, timeFilter]);

  return (
    <div className={styles.app}>
      <div className={styles.topbar}>
        <img src={logo} className={styles.appLogo} alt="logo" />
        <PlusIcon className={styles.icon} />
        <img src={btLogo} alt="bit_torrent" />
        <PlusIcon className={styles.icon} />
        <img src={crab} alt="rust" />
        <FormControl>
          <InputLabel id="filter-select-label">Filtrar peers</InputLabel>
          <Select
            id="time-select"
            value={timeFilter}
            onChange={handleTimeFilterChange}
            labelId="filter-select-label"
            className={styles.select}
          >
            <MenuItem value={null}>Todos</MenuItem>
            <MenuItem value={900}>Últimos 15 minutos</MenuItem>
            <MenuItem value={3600}>Última hora</MenuItem>
            <MenuItem value={18000}>Últimas 5 horas</MenuItem>
            <MenuItem value={86400}>Último día</MenuItem>
            <MenuItem value={259200}>Últimos 3 días</MenuItem>
          </Select>
        </FormControl>
      </div>
      <div className={styles.content}>
        <div className={styles.info}>
          {Object.keys(data).map((key) => {
            const peers = data[key].peer_list.peers || [];

            return (
              <div
                className={styles.torrentInfo}
                onClick={() => {
                  if (selectedTorrent === key) {
                    setSelectedTorrent(null);
                  } else setSelectedTorrent(key);
                }}
              >
                <span
                  key={`${key}-span`}
                  className={styles.torrentTitle}
                >{`Torrent: ${key}`}</span>
                <div
                  className={`${styles.torrentWrapper} ${
                    selectedTorrent === key && styles.showTorrents
                  }`}
                >
                  <div key={`${key}-div`} className={`${styles.torrent}`}>
                    <div className={styles.headers}>
                      {headers.map((header) => (
                        <span className={styles.header}>{header}</span>
                      ))}
                    </div>
                    {peers
                      .filter(({ created_at }) =>
                        timeFilter ? timeFilter >= created_at : true
                      )
                      .map((peer) => (
                        <div key={peer["peer id"]} className={styles.item}>
                          {Object.keys(peer).map((key) => (
                            <span className={styles.attribute}>
                              {renderColumn[key]
                                ? renderColumn[key](peer[key])
                                : peer[key]}
                            </span>
                          ))}
                        </div>
                      ))}
                  </div>
                </div>
              </div>
            );
          })}
        </div>
        {filteringOption?.graph && (
          <div className={styles.graph}>
            (
            <FormControl>
              <InputLabel id="graph-sectioning-label">Filtrar peers</InputLabel>
              <Select
                id="graph-select"
                value={graphSectioning}
                onChange={({ target }) => setGraphSectioning(target.value)}
                labelId="graph-sectioning-label"
                className={styles.select}
              >
                {filteringOption.graph.displayAs.map((option) => (
                  <MenuItem value={option}>{option}</MenuItem>
                ))}
              </Select>
            </FormControl>
            )
            {graphSectioning === "Minutos" && (
              <Bar
                id="bar-chart"
                data={{
                  labels: minuteData.map(
                    (_, index, arr) => `${arr.length - 1 - index} mins ago`
                  ),
                  datasets: [
                    {
                      backgroundColor: "#54B4D050",
                      label: "# of Peers [mins]",
                      data: minuteData
                    }
                  ]
                }}
              />
            )}
            {graphSectioning === "Horas" && (
              <Bar
                id="bar-chart"
                data={{
                  labels: hourData.map(
                    (_, index, arr) => `${arr.length - 1 - index} hs ago`
                  ),
                  datasets: [
                    {
                      backgroundColor: "#54B4D050",
                      label: "# of Peers [hs]",
                      data: hourData,
                      borderWidth: 1
                    }
                  ]
                }}
              />
            )}
          </div>
        )}
      </div>
    </div>
  );
};

export default App;
