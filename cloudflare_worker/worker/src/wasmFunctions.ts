/**
 * Copyright 2021 Google LLC
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     https://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

declare var wasm: any;
declare var wasm_bindgen: any;

type HeaderFields = Array<[string, string]>;

export interface WasmResponse {
  body: Uint8Array;
  headers: HeaderFields;
  status: number;
}

interface WasmFunctions {
  createOcspRequest(): Uint8Array;
  createRequestHeaders(fields: HeaderFields): HeaderFields;
  createSignedExchange(
    fallbackUrl: string,
    statusCode: number,
    payloadHeaders: HeaderFields,
    payloadBody: Uint8Array,
    nowInSeconds: number,
    signer: (input: Uint8Array) => Promise<Uint8Array>,
  ): WasmResponse,
  getLastErrorMessage(): string;
  servePresetContent(url: string, ocsp: Uint8Array): WasmResponse;
  shouldRespondDebugInfo(): boolean;
  validatePayloadHeaders(fields: HeaderFields): void,
}

export const wasmFunctionsPromise = (async function initWasmFunctions() {
  await wasm_bindgen(wasm);
  wasm_bindgen.init();
  return wasm_bindgen as WasmFunctions;
})();