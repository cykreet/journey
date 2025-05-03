import { Route, Switch } from "wouter";
import "./main.css";
import { Index } from "./views";
import { Course } from "./views/course";
import { GlobalLayout } from "./components/layout/global-layout";

export function App() {
	return (
		<Switch>
			<GlobalLayout>
				<Route path="/">
					<Index />
				</Route>
				<Route path="/course/:id?/:page?">
					<Course />
				</Route>
			</GlobalLayout>
			<Route>
				<p>404 not found</p>
			</Route>
		</Switch>
	);
}

export default App;
