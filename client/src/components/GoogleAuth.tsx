import jwtDecode from "jwt-decode";
import { ReactNode, createContext } from "react"
import useSessionStorageState from "use-session-storage-state";


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
  let [googleCredential, setGoogleCredential] = useSessionStorageState<string | null>("jwt", {
    defaultValue: null
  });

  let global = window as any;
  global.handleSignIn = (response: any) => {
    let credential = response.credential;
    setGoogleCredential(credential);
  }


  if (googleCredential === null) {
    return (
      <>
        <div id="g_id_onload"
          data-client_id="898187078436-49mhvq2bai7te9vjobma6sei8s68iaj9.apps.googleusercontent.com"
          data-context="signin"
          data-callback="handleSignIn"
          data-auto_select="true"
          data-use_fedcm_for_prompt="true"
          data-itp_support="true">
        </div>
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