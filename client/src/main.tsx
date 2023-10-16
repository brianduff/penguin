import React, { createRef, useEffect, useRef } from 'react'
import ReactDOM from 'react-dom/client'
import App, { AppGrid } from './App.tsx'
import './index.css'
import { ViewClient } from './ViewClient.tsx'
import { RouterProvider, createBrowserRouter } from 'react-router-dom'
import { QueryClient, QueryClientProvider } from 'react-query'
import { getClient, getClients, getDomainList, getDomainLists, getNetAccess } from './api.ts'
import { Result } from './result.ts'
import { DomainList } from './bindings/DomainList.ts'
import { NetAccess } from './bindings/NetAccess.ts'
import { Client } from './bindings/Client.ts'
import { Desktop, GlobeNetwork, Home } from '@blueprintjs/icons'
import { ViewDomains } from './ViewDomains.tsx'
import { GoogleAccountProvider } from './components/GoogleAuth.tsx'
import { css } from '@emotion/react'

const queryClient = new QueryClient();

export interface AppGridLoaderData {
  clients: Result<Client[]>,
  domains: Result<DomainList[]>
}

export interface ViewClientLoaderData {
  client: Result<Client>,
  netaccess?: Result<NetAccess>
}

const fetchClient = async (id: string) => {
  return queryClient.fetchQuery(["client", id], () => getClient(id))
}

const fetchNetAccess = async (client: Client) => {
  if (client.mac_address !== undefined) {
    let netaccess : Result<NetAccess> | undefined = await queryClient.fetchQuery(["netaccess", client.mac_address], () => getNetAccess(client.mac_address!));
    // Yicky error handling
    if (netaccess?.isErr()) {
      netaccess = undefined;
    }
    return Result.Ok({
      client: Result.Ok(client),
      netaccess
    } as ViewClientLoaderData)
  } else {
    return Result.Ok({
      client: Result.Ok(client)
    } as ViewClientLoaderData)
  }
}


const router = createBrowserRouter([
  {
    id: "root",
    path: "/",
    loader: async () => {
      const [clients, domains] = await Promise.all([
        queryClient.fetchQuery("clients", getClients),
        queryClient.fetchQuery("domainlists", getDomainLists)
      ]);

      return { clients, domains }
    },
    handle: {
      crumb: (_: any) => ({ href: "/", text: "Home", icon: <Home />})
    },
    element: <App />,
    children: [
      {
        path: "/",
        element: <AppGrid />,
        id: "appgrid",
      },
      {
        id: "client",
        path: "client/:id",
        element: <ViewClient />,
        loader: async ({ params }) => {
          return (await fetchClient(params.id!)).andThen(fetchNetAccess)
        },
        handle: {
          crumb: (data: any) => {
            let client = (data as Result<ViewClientLoaderData>).unwrap().client.unwrap();
            return ({ href: `/client/${client.id}`, text: client.name, icon: <Desktop />})
          }
        }
      },
      {
        id: "domains",
        path: "domains/:id",
        element: <ViewDomains />,
        loader: async ({ params }) => {
          return queryClient.fetchQuery("domain", () => getDomainList(params.id!))
        },
        handle: {
          crumb: (data: any) => {
            let domainList = (data as Result<DomainList>).unwrap();
            return ({ href: `/client/${domainList.id}`, text: domainList.name, icon: <GlobeNetwork />})
          }
        }
      }
    ]
  }
])

ReactDOM.createRoot(document.getElementById('root')!).render(
  <React.StrictMode>
    <GoogleAccountProvider unauthedChildren={<SplashScreen />}>
      <QueryClientProvider client={queryClient}>
        <RouterProvider router={router} />
      </QueryClientProvider>
    </GoogleAccountProvider>
  </React.StrictMode>,
)


function SplashScreen() {
  return (
    <div css={css`
      display: flex;
      justify-content: center;
      align-items: center;
      text-align: center;
      height: 100vh;
    `}>
      <div>
        <p><img src="penguin.png" css={css`max-width: 50vw`} /></p>
        <p>Use of Penguin is restricted to authorized users.</p>
        <GoogleSignInButton />
      </div>
    </div>
  )
}


function GoogleSignInButton() {
  const buttonRef = createRef<HTMLDivElement>();

  useEffect(() => {
    const element = buttonRef.current;
    if (element) {
      google.accounts.id.renderButton(element, {
        "theme": "outline",
        "size": "large",
      });
    } else {
      console.log("Element is null?")
    }
  }, [buttonRef.current]);

  return (
    <>
      <div className="GSign" css={css`text-align: center; align-items: center; display: inline-block;`} ref={buttonRef}>
      </div>
    </>
  )
}