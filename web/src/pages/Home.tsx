import { Button } from "../components/ui/button";
import {
  Card,
  CardHeader,
  CardTitle,
  CardDescription,
  CardContent,
  CardFooter,
} from "@/components/ui/card";
import { Link } from "react-router-dom";

export default function Home() {
  return (
    <div className="flex flex-col items-center justify-start min-h-[60vh] mt-8">
      <Card className="w-full max-w-3xl glass shadow-accent p-6">
        <div className="accent-stripe rounded-t-2xl mb-4" />
        <CardHeader>
          <CardTitle>
            <h1 className="text-4xl font-bold">Welcome to V.E.R.S.E</h1>
          </CardTitle>
          <CardDescription>
            <p className="mt-2 text-lg">
              Next Generation Privacy Preserving Machine Learning Platform
            </p>
          </CardDescription>
        </CardHeader>

        <CardContent>{/* Add any leading content here */}</CardContent>

        <CardFooter>
          <div className="mt-2 flex gap-3">
            <Button asChild>
              <Link to="/docs">Get Started</Link>
            </Button>
            <Button variant="outline" asChild>
              <Link to="https://github.com/Salai-Kowshikan/V.E.R.S.E">Github</Link>
            </Button>
          </div>
        </CardFooter>
      </Card>
    </div>
  );
}
