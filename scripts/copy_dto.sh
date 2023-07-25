#!/bin/bash 

BASE_DIR=$GITHUB_WORKSPACE

copy_dtos() {
  rm -rf "${BASE_DIR}/global-dtos/src/auth"
  mkdir -p "${BASE_DIR}/global-dtos/src/auth"
  cp -r "${BASE_DIR}/auth/bindings" "${BASE_DIR}/global-dtos/src/auth"
}

export_all() {
  rm -f "${BASE_DIR}/global-dtos/src/index.ts"
  DIR_STRING=$(find "${BASE_DIR}/global-dtos/src/" -type f -exec realpath --relative-to "${BASE_DIR}/global-dtos/src/" {} \;)
  DIRS=($(echo $DIR_STRING | tr " " "\n"))
  for i in "${DIRS[@]}"
  do
    NO_EXT=${i::-3}
    echo -e "export * from './${NO_EXT}'" >> "${BASE_DIR}/global-dtos/src/index.ts"
    echo $i
  done
}

copy_dtos
export_all