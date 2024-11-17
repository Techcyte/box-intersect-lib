bash build.sh /opt/python/cp312-cp312/bin/
bash build.sh /opt/python/cp311-cp311/bin/
bash build.sh /opt/python/cp310-cp310/bin/
bash build.sh /opt/python/cp39-cp39/bin/
bash build.sh /opt/python/cp38-cp38/bin/
bash build.sh /opt/python/cp37-cp37m/bin/

/opt/python/cp310-cp310/bin/python -m twine upload --repository-url $REPOSITORY_URL target/wheels/*