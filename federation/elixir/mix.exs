defmodule VailIrisFederation.MixProject do
  use Mix.Project

  @version "0.4.0"

  def project do
    [
      app:             :vail_iris_federation,
      version:         @version,
      elixir:          "~> 1.16",
      start_permanent: Mix.env() == :prod,
      deps:            deps(),
      elixirc_paths:   elixirc_paths(Mix.env()),
      aliases:         aliases(),
      # Ensure deterministic compilation for reproducible builds
      compilers:       [:elixir, :app],
      erlc_options:    [:deterministic],
    ]
  end

  def application do
    [
      extra_applications: [:logger, :crypto],
      mod: {VailIrisFederation.Application, []},
    ]
  end

  defp deps do
    [
      # Distributed PubSub for snapshot propagation
      {:phoenix_pubsub, "~> 2.1"},
      # JSON serialisation (contract envelope metadata)
      {:jason, "~> 1.4"},
      # Telemetry for SLO monitoring hooks (boundary/monitor alignment)
      {:telemetry, "~> 1.2"},
      # Test-only: property-based testing for quorum invariants
      {:stream_data, "~> 0.6", only: :test},
    ]
  end

  defp elixirc_paths(:test), do: ["lib", "test/support"]
  defp elixirc_paths(_),     do: ["lib"]

  defp aliases do
    [
      # Run tests with coverage; CI gate expects exit code 0
      test: ["test --cover"],
    ]
  end
end
