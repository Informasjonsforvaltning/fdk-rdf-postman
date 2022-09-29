for f in $(find /kafka/schemas -name '*.json'); do
    id=$(basename $f | sed "s|\.json||")
    schema=$(cat $f | jq -c | sed 's|"|\\"|g')

    curl -sSfX POST -H "Content-Type: application/vnd.schemaregistry.v1+json" \
        --data "{ \"schema\": \"${schema}\" }" \
        "${SCHEMA_REGISTRY_URL}/subjects/${id}/versions"
done

