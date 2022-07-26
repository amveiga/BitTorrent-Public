export const MOCK_TORRENT = {
  MOCK: {
    peer_list: {
      peers: new Array(200).fill(0).map((_, index) => ({
        "peer id": `MOCK_PEER_${index}`,
        port: 10 * index,
        ip: "127.0.0.1",
        event: "started",
        uploaded: 0,
        left: 100,
        created_at: Math.round(Math.random() * 259200)
      }))
    }
  }
};
