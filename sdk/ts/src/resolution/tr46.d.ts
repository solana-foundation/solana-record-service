declare module 'tr46' {
  export interface ToAsciiOptions {
    checkBidi?: boolean;
    checkHyphens?: boolean;
    checkJoiners?: boolean;
    processingOption?: 'nontransitional' | 'transitional';
    transitionalProcessing?: boolean;
    useSTD3ASCIIRules?: boolean;
    verifyDNSLength?: boolean;
  }

  export function toASCII(
    domainName: string,
    options?: ToAsciiOptions
  ): string | null;
}
