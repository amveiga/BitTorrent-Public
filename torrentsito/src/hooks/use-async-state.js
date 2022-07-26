/* eslint-disable react-hooks/exhaustive-deps */
import { debounce } from "lodash";
import { useMemo, useState } from "react";

const useAsyncState = (initialValue, asyncRequest, config) => {
  const {
    defaultLoading = false,
    failureCallback,
    debounceTime = 0
  } = config || {};

  const [inProgress, setInProgress] = useState(defaultLoading);
  const [error, setError] = useState("");
  const [data, setData] = useState(initialValue);

  const refresh = useMemo(
    () =>
      debounce((...params) => {
        setInProgress(true);
        asyncRequest(...params)
          .then((newData) => {
            setData(newData);
            setInProgress(false);
          })
          .catch((newError) => {
            setError(newError);
            failureCallback?.(newError);
          })
          .finally(() => setInProgress(false));
      }, debounceTime),
    []
  );

  return {
    data,
    refresh,
    inProgress,
    error,
    overrideState: setData
  };
};

export default useAsyncState;
