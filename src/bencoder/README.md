# Bencode
**Definition**: *Bencode* is the encoding used by the peer-to-peer file sharing system BitTorrent for storing and transmitting loosely structured data. It supports four different types of values:

- byte strings
- integers
- lists
- dictionaries

**How does it work?**

- **Strings**: length_string(b10):the_string
Example:    4:spam corresponds to 'spam'.

- **Integers**: i(Num-b10)e
Example:    i3e corresponds to 3.

- **Lists**: l(bencoded_elements)e
Example:    l4:spam4:eggse corresponds to ['spam', 'eggs']. 

- **Dictionaries**: d(bencoded_elements)e
Example:    d3:cow3:moo4:spam4:eggse corresponds to {'cow': 'moo', 'spam': 'eggs'}.
            d4:spaml1:a1:bee corresponds to {'spam': ['a', 'b']}


### **Full Example:**

**Encoding**

```
{
  string: 'Hello World',
  integer: 12345,
  dict: {
    key: 'This is a string within a dictionary'
  },
  list: [ 1, 2, 3, 4, 'string', 5, {} ]
}
```

**Output**
```
d4:dictd3:key36:This is a string within a dictionarye7:integeri12345e4:listli1ei2ei3ei4e6:stringi5edee6:string11:Hello Worlde
```

The other way round for the **Decoding.**
