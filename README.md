# taxa-py

MIBiG taxonomy handling python bindings

This package is designed to help MIBiG-related python code to handle NCBI taxid lookups using [NCBI taxdump data](https://ftp.ncbi.nlm.nih.gov/pub/taxonomy/new_taxdump/).

This package contains the python bindings for the [MIBiG taxa-rs](https://github.com/mibig-secmet/taxa-rs) package to manage a local JSON-based cache of interesting taxa, allowing bulk database imports to speed up compared to parsing directly from the taxdump files.

## Installation

To install taxa-py, run the following (assuming you are in a python virtualenv):

```
pip install mibig-taxa
```

## Usage

To create a cache file, first grab the [latest taxdump collection](https://ftp.ncbi.nlm.nih.gov/pub/taxonomy/new_taxdump/) and extract it. You'll also need a directory containing the MIBiG BGC entry JSON files.

Then run the following:

```python
from mibig_taxa import TaxonCache

cache = TaxonCache()
cache.initialise(
    taxdump="path/to/taxa/rankedlineage.dmp",
    merged_id_dump="path/to/taxa/merged.dmp",
    datadir="path/to/mibig-json/data"
)

# Save the cache to a file for later use
cache.save("my_cache.json")
```

If you want to use the cache in a different process, simply load the cache like this:

```python
from mibig_taxa import TaxonCache

cache = TaxonCache("my_cache.json")

# Or, if you prefer the longer form
cache = TaxonCache()
cache.load("my_cache.json")

```

To get an ID mapping, use

```python
from mibig_taxa import TaxonCache

cache = TaxonCache("my_cache.json")

id_to_map = 123456
name = get_name_by_id(id_to_map)

print(f"Taxon with ID {id_to_map} is called {name}")
```

If you want to transparently support deprecated IDs, also set the `allow_deprecated` argument to `True`:

```python
from mibig_taxa import TaxonCache

cache = TaxonCache("my_cache.json")

deprecated_id_to_map = 123456
name = get_name_by_id(deprecated_id_to_map, allow_deprecated=True)

print(f"Taxon with deprecated ID {deprecated_id_to_map} is called {name}")
```

## License

Licensed under the Apache License, Version 2.0
([LICENSE](LICENSE) or http://www.apache.org/licenses/LICENSE-2.0)


## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
licensed as Apache-2.0, without any additional terms or conditions.
