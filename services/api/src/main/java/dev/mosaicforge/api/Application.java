package dev.mosaicforge.api;

import org.springframework.boot.SpringApplication;
import org.springframework.boot.autoconfigure.SpringBootApplication;
import org.springframework.context.annotation.Bean;
import org.springframework.core.env.Environment;

@SpringBootApplication
public class Application {
  public static void main(String[] args) {
    SpringApplication.run(Application.class, args);
  }

  @Bean
  Contracts contracts() {
    return new Contracts();
  }

  @Bean
  EnginePort engine(Environment environment) {
    return new HttpEngine(
        environment.getRequiredProperty("mosaicforge.engine-url"),
        environment.getRequiredProperty("mosaicforge.engine-token"),
        environment.getProperty("mosaicforge.timeout-ms", Long.class, 30000L));
  }
}
