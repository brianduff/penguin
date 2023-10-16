import { CredentialResponse, IdConfiguration } from "google-one-tap";
import jwtDecode from "jwt-decode";
import { ReactNode, createContext } from "react"
import useSessionStorageState from "use-session-storage-state";

var googleCredential : string | null = null;
var handleSignIn = function(r: CredentialResponse) {
  console.log("Obtained Google credential");
  googleCredential = r.credential;
  window.sessionStorage.setItem("jwt", googleCredential);
}

function getGoogleCredential() {
  if (googleCredential) {
    return googleCredential;
  }
  let sessionJwt = window.sessionStorage.getItem("jwt");
  if (sessionJwt) {
    return sessionJwt;
  }
  return null;
}

window.onload = function() {
  let options = {
    auto_select: true,
    client_id: "898187078436-49mhvq2bai7te9vjobma6sei8s68iaj9.apps.googleusercontent.com",
    callback: handleSignIn,
    use_fedcm_for_prompt: true,
    itp_support: true,
    context: "signin"
  }
  google.accounts.id.initialize(options as IdConfiguration)
  let credential = getGoogleCredential();
  if (credential === null || credential === undefined) {
    google.accounts.id.prompt();
  }
  console.log("Initialized Google auth")
}

export interface GoogleCredentials {
  iss: string,
  nbf: number,
  aud: string,
  azp: string,
  email: string,
  email_verified: boolean,
  exp: number,
  family_name: string,
  given_name: string,
  iat: number,
  jti: string,
  name: string,
  picture: string,
  sub: string
}

export const CredentialsContext = createContext<null | GoogleCredentials>(null);

interface Kids {
  unauthedChildren?: ReactNode,
  children?: ReactNode
}
export function GoogleAccountProvider({ unauthedChildren, children }: Kids) {
  let googleCredential = getGoogleCredential();
  if (googleCredential === null || googleCredential === undefined) {
    return (
      <>
        {unauthedChildren}
      </>
    )
  } else {
    console.log("Rendering authenticated flow with ", googleCredential);

    // Decode the token.
    let credentials: GoogleCredentials = jwtDecode(googleCredential);

    return (
      <CredentialsContext.Provider value={credentials}>
        {children}
      </CredentialsContext.Provider>
    );
  }
}