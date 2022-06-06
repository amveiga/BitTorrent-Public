# Torrent File

A **torrent file** contains a list of files and integrity metadata about all the pieces, and optionally contains a large list of trackers.

It is a bencoded dictionary with the following keys (the keys in any bencoded dictionary are lexicographically ordered):

- **announce:** The URL of the tracker
  
- **info:** This maps to a dictionary whose keys are dependent on whether **one or more files are being shared**:
   - ***files:*** a list of dictionaries each corresponding to a file (only when multiple files are being shared). Each dictionary has the following keys:
     - ***length:*** size of the file in bytes.
  
     - ***path:*** a list of strings corresponding to subdirectory names, the last of which is the actual file name
  

  - **length:** size of the file in bytes (only when one file is being shared though)


  - **name:** suggested filename where the file is to be saved (if one file)/suggested directory name where the files are to be saved (if multiple files)
  
  - **piece length:** number of bytes per piece. This is commonly 28 KiB = 256 KiB = 262,144 B.
  
  - **pieces:** a hash list, i.e., a concatenation of each piece's SHA-1 hash. As SHA-1 returns a 160-bit hash, pieces will be a string whose length is a multiple of 20 bytes. If the torrent contains multiple files, the pieces are formed by concatenating the files in the order they appear in the filesdictionary (i.e., all pieces in the torrent are the full piece length except for the last piece, which may be shorter).
  

All strings must be UTF-8 encoded, except for pieces, which contains binary data.

A torrent is uniquely identified by an infohash, a SHA-1 hash calculated over the contents of the info dictionary in bencode form. Changes to other portions of the torrent does not affect the hash. This hash is used to identify the torrent to other peers via DHT and to the tracker. It is also used in magnet links.