# MQTT Simulator

This is a small tool to simulate a MQTT publisher with data taken from a JSON file. The published
data is continuously updated by monitoring changes to the data file.

Data can be published in a few formats: bool, int, float and String with some customization, e.g. all
native integer and float types are supported in any width and endianness, Strings can be published as
UTF8, UTF16LE and UTF16BE. Arrays of those primitives can also be defined, the data points are simply
concatenated. Finally, arbitrary JSON objects are supported.

## Usage

Set up data in `data.json`

~~~JSON
[
    {
        "topic": "fake_str",
        "data": {
            "value": "hello world",
            "encoding": "UTF16LE"
        }
    },
    {
        "topic": "stringint",
        "data": {
            "value": "2"
        }
    },
    {
        "topic": "stringfloat",
        "data": {
            "value": "2"
        }
    },
    {
        "topic": "stringbool",
        "data": {
            "value": "true"
        }
    },
    {
        "topic": "u32_le",
        "data": {
            "value": 10,
            "width": "32",
            "endian": "LittleEndian"
        }
    },
    {
        "topic": "i32_le",
        "data": {
            "value": -10,
            "width": "32",
            "endian": "LittleEndian"
        }
    },
    {
        "topic": "f32_be",
        "data": {
            "value": 2.3,
            "width": "32",
            "endian": "BigEndian"
        }
    },
    {
        "topic": "bool",
        "data": false
    },
    {
        "topic": "json",
        "data": {
            "mapping": {
                "nested": true,
            },
            "n_fields": 2
        }
    },
    {
        "topic": "array",
        "data": [
            {
                "value": 1
            },
            {
                "value": "heterogeneous"
            }
        ]
    }
]
~~~

Run the publisher with

~~~bash
# only the data file argument is required, rest is populated with defaults.
mqtt-simulator data.json --host localhost --port 1883 --send-interval 1000 --client-id fake-publisher
~~~

Edit the data as desired, the tool automatically refreshes its data once changes are detected.

## Data

All entries in the list designate the topic they are published under in the `topic` field. The actual
data is placed in the `data` field. The format and the keys for the `data` field differ depending on
the data type.

### Boolean

Booleans have a single field with `true` or `false` as possible values.
~~~JSON
{
    "topic": "cool_bool",
    "data": true
}
~~~

### Integer

Integers are specified with 3 fields, `value`, `width` and `endian`.

Only values `x < 0 ` are treated as signed integers, all other values are treated as unsigned.

`width` determines the width of the published integer in bits, `endian` the endianness of the published
message. `width` defaults to `64` and `endian` to `BigEndian` if left out. The possible values are
listed below:

**Endian:**
  * `"BigEndian"`
  * `"LittleEndian"`

**Width:**
  * `"8"`
  * `"16"`
  * `"32"`
  * `"64"`


**Examples:**

Publish an unsigned big endian 64bit wide value:

~~~JSON
{
    "topic": "uint",
    "data": {
        "value": 2
    }
}
~~~

Publish an unsigned little endian 64bit wide value:

~~~JSON
{
    "topic": "uint",
    "data": {
        "value": 2,
        "endian": "LittleEndian"
    }
}
~~~

Publish an unsigned little endian 32bit wide value:

~~~JSON
{
    "topic": "uint",
    "data": {
        "value": 2,
        "endian": "LittleEndian",
        "width": "32"
    }
}
~~~

### Float

Floats behave very similar to Integers, also being specified by `value`, `width` and `endian` fields.

`width` is more restricted and only allows for the natively supported 32 and 64 bit variants. It defaults
to `"64"`.

`endian` has the same choices as listed above.

### String

Strings can be published as `UTF8`, `UTF16LE` and `UTF16BE`, specification is similar to the other data
types. The default is `UTF8`.

**Examples**

Publish a UTF8 String:
~~~JSON
{
    "topic": "utf8_string",
    "data": {
        "value": "hello world",
    }
}
~~~

Publish a UTF16LE String:

~~~JSON
{
    "topic": "utf8_string",
    "data": {
        "value": "hello world",
        "encoding": "UTF16LE"
    }
}
~~~

Publish a UTF16BE String:

~~~JSON
{
    "topic": "utf8_string",
    "data": {
        "value": "hello world",
        "encoding": "UTF16BE"
    }
}
~~~

### Arrays

Heterogeneous, possibly nested, arrays can be defined as arrays of `Value`s. The array is simply a container
that does not influence the serialization of the contained values. There are no delimiters, size fields or
anything else added, the payloads of the contained values are simply concatenated.

**Examples**

~~~JSON
{
    "topic": "array",
    "data": [
        {
            "value": "some string"
        },
        {
            "value": 12
        },
        [
            {
                "value": "it's nested!"
            }
        ]
    ]
}
~~~

### JSON

To ensure structural integrity of JSON strings, JSON objects are supported via `serde_json`. Data is published
based on the output of `serde_json::to_writer`.

**Examples**

~~~JSON
{
    "topic": "json_object",
    "data": {
        "mapping": {
            "nested": true
        },
        "array": ["anything goes", 2, false],
        "string": "value"
    }
}
~~~